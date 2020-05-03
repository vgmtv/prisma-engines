use crate::database::connection::CachedConnection;
use async_trait::async_trait;
use connector_interface::{Connection, Connector, IO};
use datamodel::Source;
// use lru_cache::LruCache;
use prisma_models::ModelRef;
use prisma_value::PrismaValue;
use sql_query_connector::{FromSource, PostgreSql, SqlError};
use std::collections::HashSet;
use std::sync::Mutex;

//Todo Cache Structures
//start with only three caches.
// MODEL   HashSet
// MODELFILTER   Cache(model, filter -> id)
// FROMFIELD     Cache(from_field , from_id -> vec![target_id])
// idea: cache of deleted nodes
pub struct ModelCache {
    model: Mutex<HashSet<(ModelRef, PrismaValue)>>,
}

impl ModelCache {
    pub fn get(&self, model: ModelRef, id: PrismaValue) -> bool {
        let lock = self.model.lock();
        lock.unwrap().contains(&(model, id))
    }

    pub fn insert(&self, model: ModelRef, id: PrismaValue) {
        let lock = self.model.lock();
        lock.unwrap().insert((model, id));
    }
}

pub struct Cached {
    cache: ModelCache,
    inner: PostgreSql,
}

#[async_trait]
impl FromSource for Cached {
    async fn from_source(source: &dyn Source) -> connector_interface::Result<Self> {
        let psql = PostgreSql::from_source(source).await?;
        let cache = ModelCache {
            model: Mutex::new(HashSet::with_capacity(10)),
        };

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
