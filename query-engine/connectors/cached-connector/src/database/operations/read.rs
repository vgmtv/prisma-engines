use crate::Cache;
use connector_interface::*;
use prisma_models::*;
use sql_query_connector::operations::read;
use sql_query_connector::QueryExt;

pub async fn get_single_record(
    conn: &dyn QueryExt,
    cache: &Cache,
    model: &ModelRef,
    filter: &Filter,
    selected_fields: &ModelProjection,
) -> crate::Result<Option<SingleRecord>> {
    read::get_single_record(conn, model, filter, selected_fields).await
    //MODEL (MODEL) -> vec![ID]
    //MODELFILTER (MODEL, FILTER) -> ID

    //retrieve from cache:
    //If Select (only ID) from MODEL Where id=id -> check in MODEL
    //If Select (only ID) from MODEL Where FILTER -> check in MODELFILTER

    //store result in cache if not already in there
    //update MODEL, MODELFILTER
}

pub async fn get_many_records(
    conn: &dyn QueryExt,
    model: &ModelRef,
    query_arguments: QueryArguments,
    selected_fields: &ModelProjection,
) -> crate::Result<ManyRecords> {
    read::get_many_records(conn, model, query_arguments, selected_fields).await
    //Dont cache yet
}

pub async fn get_related_m2m_record_ids(
    conn: &dyn QueryExt,
    from_field: &RelationFieldRef,
    from_record_ids: &[RecordProjection],
) -> crate::Result<Vec<(RecordProjection, RecordProjection)>> {
    read::get_related_m2m_record_ids(conn, from_field, from_record_ids).await
    //FROMFIELD     (FROM_FIELD , FROM_ID -> VEC![TARGET_ID])

    //retrieve from cache:
    //If Select (FROM_FIELD) from MODEL Where FROM_ID

    //store in cache
    //FROM_FIELD , FROM_ID -> VEC![TARGET_ID]
}

pub async fn count_by_model(
    conn: &dyn QueryExt,
    model: &ModelRef,
    query_arguments: QueryArguments,
) -> crate::Result<usize> {
    read::count_by_model(conn, model, query_arguments).await
    //Dont cache yet
}
