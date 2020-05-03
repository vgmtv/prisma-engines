use crate::database::operations::*;
use crate::ModelCache;
use connector_interface::{
    self as connector, error::*, filter::Filter, QueryArguments, ReadOperations, RecordFilter, Transaction, WriteArgs,
    WriteOperations, IO,
};
use prisma_models::prelude::*;
use prisma_value::PrismaValue;
use quaint::prelude::ConnectionInfo;
use sql_query_connector::SqlError;

pub struct CachedConnectorTransaction<'a> {
    cache: &'a ModelCache,
    inner: quaint::connector::Transaction<'a>,
    connection_info: &'a ConnectionInfo,
}

impl<'a> CachedConnectorTransaction<'a> {
    pub fn new<'b: 'a>(
        cache: &'a ModelCache,
        tx: quaint::connector::Transaction<'a>,
        connection_info: &'b ConnectionInfo,
    ) -> Self {
        Self {
            cache,
            inner: tx,
            connection_info,
        }
    }

    async fn catch<O>(&self, fut: impl std::future::Future<Output = Result<O, SqlError>>) -> Result<O, ConnectorError> {
        match fut.await {
            Ok(o) => Ok(o),
            Err(err) => Err(ConnectorError::from_kind(ErrorKind::ColumnDoesNotExist)),
        }
    }
}

impl<'a> Transaction<'a> for CachedConnectorTransaction<'a> {
    fn commit<'b>(&'b self) -> IO<'b, ()> {
        IO::new(self.catch(async move { Ok(self.inner.commit().await.map_err(SqlError::from)?) }))
    }

    fn rollback<'b>(&'b self) -> IO<'b, ()> {
        IO::new(self.catch(async move { Ok(self.inner.rollback().await.map_err(SqlError::from)?) }))
    }
}

impl<'a> ReadOperations for CachedConnectorTransaction<'a> {
    fn get_single_record<'b>(
        &'b self,
        model: &'b ModelRef,
        filter: &'b Filter,
        selected_fields: &'b ModelProjection,
    ) -> connector::IO<'b, Option<SingleRecord>> {
        IO::new(self.catch(async move {
            read::get_single_record(&self.inner, self.cache, model, filter, selected_fields).await
        }))
    }

    fn get_many_records<'b>(
        &'b self,
        model: &'b ModelRef,
        query_arguments: QueryArguments,
        selected_fields: &'b ModelProjection,
    ) -> connector::IO<'b, ManyRecords> {
        IO::new(
            self.catch(
                async move { read::get_many_records(&self.inner, model, query_arguments, selected_fields).await },
            ),
        )
    }

    fn get_related_m2m_record_ids<'b>(
        &'b self,
        from_field: &'b RelationFieldRef,
        from_record_ids: &'b [RecordProjection],
    ) -> connector::IO<'b, Vec<(RecordProjection, RecordProjection)>> {
        IO::new(
            self.catch(async move { read::get_related_m2m_record_ids(&self.inner, from_field, from_record_ids).await }),
        )
    }

    fn count_by_model<'b>(&'b self, model: &'b ModelRef, query_arguments: QueryArguments) -> connector::IO<'b, usize> {
        IO::new(self.catch(async move { read::count_by_model(&self.inner, model, query_arguments).await }))
    }
}

impl<'a> WriteOperations for CachedConnectorTransaction<'a> {
    fn create_record<'b>(&'b self, model: &'b ModelRef, args: WriteArgs) -> connector::IO<RecordProjection> {
        IO::new(self.catch(async move { write::create_record(&self.inner, model, args).await }))
    }

    fn update_records<'b>(
        &'b self,
        model: &'b ModelRef,
        record_filter: RecordFilter,
        args: WriteArgs,
    ) -> connector::IO<Vec<RecordProjection>> {
        IO::new(self.catch(async move { write::update_records(&self.inner, model, record_filter, args).await }))
    }

    fn delete_records<'b>(&'b self, model: &'b ModelRef, record_filter: RecordFilter) -> connector::IO<usize> {
        IO::new(self.catch(async move { write::delete_records(&self.inner, model, record_filter).await }))
    }

    fn connect<'b>(
        &'b self,
        field: &'b RelationFieldRef,
        parent_id: &'b RecordProjection,
        child_ids: &'b [RecordProjection],
    ) -> connector::IO<()> {
        IO::new(self.catch(async move { write::connect(&self.inner, field, parent_id, child_ids).await }))
    }

    fn disconnect<'b>(
        &'b self,
        field: &'b RelationFieldRef,
        parent_id: &'b RecordProjection,
        child_ids: &'b [RecordProjection],
    ) -> connector::IO<()> {
        IO::new(self.catch(async move { write::disconnect(&self.inner, field, parent_id, child_ids).await }))
    }

    fn execute_raw(&self, query: String, parameters: Vec<PrismaValue>) -> connector::IO<serde_json::Value> {
        IO::new(self.catch(async move { write::execute_raw(&self.inner, query, parameters).await }))
    }
}