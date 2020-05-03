use connector_interface::*;
use prisma_models::*;
use sql_query_connector::operations::write;
use sql_query_connector::QueryExt;

/// Create a single record to the database defined in `conn`, resulting into a
/// `RecordProjection` as an identifier pointing to the just-created record.
pub async fn create_record(conn: &dyn QueryExt, model: &ModelRef, args: WriteArgs) -> crate::Result<RecordProjection> {
    write::create_record(conn, model, args).await

    // update MODEL
}

/// Update multiple records in a database defined in `conn` and the records
/// defined in `args`, resulting the identifiers that were modified in the
/// operation.
pub async fn update_records(
    conn: &dyn QueryExt,
    model: &ModelRef,
    record_filter: RecordFilter,
    args: WriteArgs,
) -> crate::Result<Vec<RecordProjection>> {
    write::update_records(conn, model, record_filter, args).await

    //invalidate MODELFILTER for all that contain any of the returned ids
}

/// Delete multiple records in `conn`, defined in the `Filter`. Result is the number of items deleted.
pub async fn delete_records(
    conn: &dyn QueryExt,
    model: &ModelRef,
    record_filter: RecordFilter,
) -> crate::Result<usize> {
    write::delete_records(conn, model, record_filter).await
    //this should also return RecordProjections and delete the ids from all their usages
    //wipe from MODEL, MODELFILTER, FROMFIELD
}

/// Connect relations defined in `child_ids` to a parent defined in `parent_id`.
/// The relation information is in the `RelationFieldRef`.
pub async fn connect(
    conn: &dyn QueryExt,
    field: &RelationFieldRef,
    parent_id: &RecordProjection,
    child_ids: &[RecordProjection],
) -> crate::Result<()> {
    write::connect(conn, field, parent_id, child_ids).await
    //add to FROMFIELD
}

/// Disconnect relations defined in `child_ids` to a parent defined in `parent_id`.
/// The relation information is in the `RelationFieldRef`.
pub async fn disconnect(
    conn: &dyn QueryExt,
    field: &RelationFieldRef,
    parent_id: &RecordProjection,
    child_ids: &[RecordProjection],
) -> crate::Result<()> {
    write::disconnect(conn, field, parent_id, child_ids).await
    //remove from FROMFIELD
}

/// Execute a plain SQL query with the given parameters, returning the answer as
/// a JSON `Value`.
pub async fn execute_raw(
    conn: &dyn QueryExt,
    query: String,
    parameters: Vec<PrismaValue>,
) -> crate::Result<serde_json::Value> {
    write::execute_raw(conn, query, parameters).await
}
