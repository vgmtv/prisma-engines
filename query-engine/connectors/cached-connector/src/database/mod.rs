mod cached;
mod connection;
mod transaction;

pub(crate) mod operations;

use async_trait::async_trait;
pub use cached::*;
use connector_interface::{error::*, Connector};
use datamodel::Source;

#[async_trait]
pub trait FromSource {
    async fn from_source(source: &dyn Source) -> connector_interface::Result<Self>
    where
        Self: Connector + Sized;
}

async fn catch<O>(
    connection_info: &quaint::prelude::ConnectionInfo,
    fut: impl std::future::Future<Output = Result<O, crate::SqlError>>,
) -> Result<O, ConnectorError> {
    match fut.await {
        Ok(o) => Ok(o),
        Err(err) => Err(ConnectorError::from_kind(ErrorKind::ColumnDoesNotExist)),
    }
}
