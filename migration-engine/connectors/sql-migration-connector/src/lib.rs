mod component;
mod database_info;
mod datamodel_helpers;
mod error;
mod flavour;
mod sql_database_migration_inferrer;
mod sql_database_step_applier;
mod sql_destructive_changes_checker;
mod sql_migration;
mod sql_migration_persistence;
mod sql_renderer;
mod sql_schema_calculator;
mod sql_schema_differ;
mod sql_schema_helpers;

pub use error::*;
pub use sql_migration::*;
pub use sql_migration_persistence::MIGRATION_TABLE_NAME;

use component::Component;
use database_info::DatabaseInfo;
use flavour::SqlFlavour;
use migration_connector::*;
use quaint::{
    error::ErrorKind,
    prelude::{ConnectionInfo, Queryable, SqlFamily},
    single::Quaint,
};
use sql_database_migration_inferrer::*;
use sql_database_step_applier::*;
use sql_destructive_changes_checker::*;
use sql_migration_persistence::*;
use sql_schema_describer::SqlSchema;
use std::{sync::Arc, time::Duration};
use tracing::debug;
use user_facing_errors::migration_engine::DatabaseMigrationFormatChanged;

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

pub struct SqlMigrationConnector {
    pub database: Arc<dyn Queryable + Send + Sync + 'static>,
    pub database_info: DatabaseInfo,
    flavour: Box<dyn SqlFlavour + Send + Sync + 'static>,
}

impl SqlMigrationConnector {
    pub async fn new(database_str: &str) -> ConnectorResult<Self> {
        let connection_info =
            ConnectionInfo::from_url(database_str).map_err(|err| ConnectorError::url_parse_error(err, database_str))?;

        let connection_fut = async {
            let connection = Quaint::new(database_str)
                .await
                .map_err(SqlError::from)
                .map_err(|err: SqlError| err.into_connector_error(&connection_info))?;

            // async connections can be lazy, so we issue a simple query to fail early if the database
            // is not reachable.
            connection
                .query_raw("SELECT 1", &[])
                .await
                .map_err(SqlError::from)
                .map_err(|err| err.into_connector_error(&connection.connection_info()))?;

            Ok::<_, ConnectorError>(connection)
        };

        let connection = tokio::time::timeout(CONNECTION_TIMEOUT, connection_fut)
            .await
            .map_err(|_elapsed| {
                SqlError::from(ErrorKind::ConnectTimeout("Tokio timer".into())).into_connector_error(&connection_info)
            })??;

        let database_info = DatabaseInfo::new(&connection, connection.connection_info().clone())
            .await
            .map_err(|sql_error| sql_error.into_connector_error(&connection_info))?;

        let flavour = flavour::from_database_info(&database_info);
        flavour.check_database_info(&database_info)?;

        let conn = Arc::new(connection) as Arc<dyn Queryable + Send + Sync>;

        Ok(Self {
            flavour,
            database_info,
            database: conn,
        })
    }

    async fn drop_database(&self) -> ConnectorResult<()> {
        use quaint::ast::Value;

        catch(self.database_info.connection_info(), async {
            match &self.database_info.connection_info() {
                ConnectionInfo::Postgres(_) => {
                    let sql_str = format!(r#"DROP SCHEMA "{}" CASCADE;"#, self.schema_name());
                    debug!("{}", sql_str);

                    self.conn().query_raw(&sql_str, &[]).await.ok();
                }
                ConnectionInfo::Sqlite { file_path, .. } => {
                    self.conn()
                        .query_raw("DETACH DATABASE ?", &[Value::from(self.schema_name())])
                        .await
                        .ok();
                    std::fs::remove_file(file_path).ok(); // ignore potential errors
                    self.conn()
                        .query_raw(
                            "ATTACH DATABASE ? AS ?",
                            &[Value::from(file_path.as_str()), Value::from(self.schema_name())],
                        )
                        .await?;
                }
                ConnectionInfo::Mysql(_) => {
                    let sql_str = format!(r#"DROP SCHEMA `{}`;"#, self.schema_name());
                    debug!("{}", sql_str);
                    self.conn().query_raw(&sql_str, &[]).await?;
                }
            };

            Ok(())
        })
        .await?;

        Ok(())
    }

    async fn describe(&self) -> SqlResult<SqlSchema> {
        let conn = self.connector().database.clone();
        let schema_name = self.schema_name();

        self.flavour.describe_schema(schema_name, conn).await
    }
}

#[async_trait::async_trait]
impl MigrationConnector for SqlMigrationConnector {
    type DatabaseMigration = SqlMigration;

    fn connector_type(&self) -> &'static str {
        self.database_info.connection_info().sql_family().as_str()
    }

    async fn create_database(&self, db_name: &str) -> ConnectorResult<()> {
        catch(
            self.database_info.connection_info(),
            self.flavour.create_database(db_name, self.database.as_ref()),
        )
        .await
    }

    async fn initialize(&self) -> ConnectorResult<()> {
        catch(
            self.database_info.connection_info(),
            self.flavour.initialize(self.database.as_ref(), &self.database_info),
        )
        .await?;

        self.migration_persistence().init().await?;

        Ok(())
    }

    async fn reset(&self) -> ConnectorResult<()> {
        self.migration_persistence().reset().await?;
        self.drop_database().await?;

        Ok(())
    }

    /// Optionally check that the features implied by the provided datamodel are all compatible with
    /// the specific database version being used.
    fn check_database_version_compatibility(&self, datamodel: &datamodel::dml::Datamodel) -> Vec<MigrationError> {
        self.database_info.check_database_version_compatibility(datamodel)
    }

    fn migration_persistence<'a>(&'a self) -> Box<dyn MigrationPersistence + 'a> {
        Box::new(SqlMigrationPersistence { connector: self })
    }

    fn database_migration_inferrer<'a>(&'a self) -> Box<dyn DatabaseMigrationInferrer<SqlMigration> + 'a> {
        Box::new(SqlDatabaseMigrationInferrer { connector: self })
    }

    fn database_migration_step_applier<'a>(&'a self) -> Box<dyn DatabaseMigrationStepApplier<SqlMigration> + 'a> {
        Box::new(SqlDatabaseStepApplier { connector: self })
    }

    fn destructive_changes_checker<'a>(&'a self) -> Box<dyn DestructiveChangesChecker<SqlMigration> + 'a> {
        Box::new(SqlDestructiveChangesChecker { connector: self })
    }

    fn deserialize_database_migration(
        &self,
        json: serde_json::Value,
    ) -> Result<SqlMigration, DatabaseMigrationFormatChanged> {
        serde_json::from_value(json).map_err(|_| DatabaseMigrationFormatChanged)
    }
}

pub(crate) async fn catch<O>(
    connection_info: &ConnectionInfo,
    fut: impl std::future::Future<Output = Result<O, SqlError>>,
) -> Result<O, ConnectorError> {
    match fut.await {
        Ok(o) => Ok(o),
        Err(sql_error) => Err(sql_error.into_connector_error(connection_info)),
    }
}
