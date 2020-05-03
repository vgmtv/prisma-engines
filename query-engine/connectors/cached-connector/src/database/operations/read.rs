use crate::ModelCache;
use connector_interface::*;
use prisma_models::*;
use sql_query_connector::operations::read;
use sql_query_connector::QueryExt;

pub async fn get_single_record(
    conn: &dyn QueryExt,
    cache: &ModelCache,
    model: &ModelRef,
    filter: &Filter,
    selected_fields: &ModelProjection,
) -> crate::Result<Option<SingleRecord>> {
    //filter is by id?
    //modelprojection was only id

    let fields = model.fields();
    let id_fields = model.fields().id().unwrap();
    let id_field = id_fields.first().unwrap();

    if let Filter::Scalar(ScalarFilter {
        projection: ScalarProjection::Single(id_field_projection),
        condition: ScalarCondition::Equals(id_value),
    }) = filter
    {
        if id_field_projection == id_field {
            if selected_fields.fields().count() == 1
                && selected_fields
                    .fields()
                    .find(|f| **f == Field::Scalar(id_field.clone()))
                    .is_some()
            {
                if cache.get(model.clone(), id_value.clone()) {
                    println!("CACHE HIT");

                    return crate::Result::Ok(Some(SingleRecord {
                        record: Record::new(vec![id_value.clone()]),
                        field_names: vec![id_field.name.clone()],
                    }));
                }
            }
        }
    }

    let return_value = read::get_single_record(conn, model, filter, selected_fields).await;

    if let Ok(Some(SingleRecord {
        record: Record { values, .. },
        field_names,
    })) = return_value.as_ref()
    {
        if let Some(pos) = field_names.iter().position(|name| *name == id_field.name) {
            cache.insert(model.clone(), values[pos].clone())
        }
    }

    //fill cache

    return_value

    //MODEL (MODEL) -> vec![ID]
    //MODELFILTER (MODEL, FILTER) -> ID

    //retrieve from cache:
    //If Select (ModelProjection only ID) from MODEL Where id=id -> check in MODEL
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
    println!("GET MANY");

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
