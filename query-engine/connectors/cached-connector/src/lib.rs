pub use database::*;
use sql_query_connector::SqlError;
mod database;

pub use database::*;

type Result<T> = std::result::Result<T, SqlError>;
