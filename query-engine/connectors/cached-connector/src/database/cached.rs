use async_trait::async_trait;
use connector_interface::{Connection, Connector, IO};
use datamodel::Source;
use lru_cache::LruCache;
use prisma_models::{ModelRef, RecordProjection};

use crate::database::connection::CachedConnection;
use sql_query_connector::{FromSource, PostgreSql, SqlError};

//Todo Cache Structures
//start with only three caches.
// MODEL   Cache(model -> vec![id])
// MODELFILTER   Cache(model, filter -> id)
// FROMFIELD     Cache(from_field , from_id -> vec![target_id])
pub struct Cache {}

pub struct Cached {
    cache: Cache,
    inner: PostgreSql,
}

#[async_trait]
impl FromSource for Cached {
    async fn from_source(source: &dyn Source) -> connector_interface::Result<Self> {
        let psql = PostgreSql::from_source(source).await?;
        let cache = Cache {};

        Ok(Cached { cache, inner: psql })
    }
}

impl Connector for Cached {
    fn get_connection<'a>(&'a self) -> IO<Box<dyn Connection + 'a>> {
        IO::new(super::catch(&self.inner.connection_info, async move {
            let conn = self.inner.pool.check_out().await.map_err(SqlError::from)?;
            let conn = CachedConnection::new(&self.cache, conn, &self.inner.connection_info);

            Ok(Box::new(conn) as Box<dyn Connection>)
        }))
    }
}
