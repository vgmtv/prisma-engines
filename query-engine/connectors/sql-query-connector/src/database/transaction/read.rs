use super::SqlConnectorTransaction;
use crate::{
    query_builder::read::{ManyRelatedRecordsBaseQuery, ManyRelatedRecordsQueryBuilder, ReadQueryBuilder},
    QueryExt, SqlError,
};
use connector_interface::{error::ConnectorError, *};
use itertools::Itertools;
use prisma_models::*;
use std::convert::TryFrom;

struct ScalarListElement {
    record_id: GraphqlId,
    value: PrismaValue,
}

impl<T> ReadOperations for SqlConnectorTransaction<'_, T>
where
    T: ManyRelatedRecordsQueryBuilder,
{
    fn get_single_record(
        &mut self,
        record_finder: &RecordFinder,
        selected_fields: &SelectedFields,
    ) -> connector_interface::Result<Option<SingleRecord>> {
        let query = ReadQueryBuilder::get_records(record_finder.field.model(), selected_fields, record_finder);
        let field_names = selected_fields.names();
        let idents = selected_fields.type_identifiers();

        let record = (match self.inner.find(query, idents.as_slice()) {
            Ok(result) => Ok(Some(result)),
            Err(_e @ SqlError::RecordNotFoundForWhere(_)) => Ok(None),
            Err(e) => Err(e),
        })?
        .map(Record::from)
        .map(|record| SingleRecord { record, field_names });

        Ok(record)
    }

    fn get_many_records(
        &mut self,
        model: ModelRef,
        query_arguments: QueryArguments,
        selected_fields: &SelectedFields,
    ) -> connector_interface::Result<ManyRecords> {
        let field_names = selected_fields.names();
        let idents = selected_fields.type_identifiers();
        let query = ReadQueryBuilder::get_records(model, selected_fields, query_arguments);

        let records = self
            .inner
            .filter(query.into(), idents.as_slice())?
            .into_iter()
            .map(Record::from)
            .collect();

        Ok(ManyRecords { records, field_names })
    }

    fn get_related_records(
        &mut self,
        from_field: RelationFieldRef,
        from_record_ids: &[GraphqlId],
        query_arguments: QueryArguments,
        selected_fields: &SelectedFields,
    ) -> connector_interface::Result<ManyRecords> {
        let idents = selected_fields.type_identifiers();
        let field_names = selected_fields.names();

        let query = {
            let is_with_pagination = query_arguments.is_with_pagination();
            let base = ManyRelatedRecordsBaseQuery::new(from_field, from_record_ids, query_arguments, selected_fields);

            if is_with_pagination {
                T::with_pagination(base)
            } else {
                T::without_pagination(base)
            }
        };

        let records: Result<Vec<Record>> = self
            .inner
            .filter(query, idents.as_slice())?
            .into_iter()
            .map(|mut row| {
                let parent_id = row.values.pop().ok_or(ConnectorError::ColumnDoesNotExist)?;

                // Relation id is always the second last value. We don't need it
                // here and we don't need it in the record.
                let _ = row.values.pop();

                let mut record = Record::from(row);
                record.add_parent_id(GraphqlId::try_from(parent_id)?);

                Ok(record)
            })
            .collect();

        Ok(ManyRecords {
            records: records?,
            field_names,
        })
    }

    fn get_scalar_list_values(
        &mut self,
        list_field: ScalarFieldRef,
        record_ids: Vec<GraphqlId>,
    ) -> connector_interface::Result<Vec<ScalarListValues>> {
        let type_identifier = list_field.type_identifier;
        let query = ReadQueryBuilder::get_scalar_list_values_by_record_ids(list_field, record_ids);
        let rows = self
            .inner
            .filter(query.into(), &[TypeIdentifier::GraphQLID, type_identifier])?;

        let results: Vec<ScalarListElement> = rows
            .into_iter()
            .map(|row| {
                let mut iter = row.values.into_iter();

                let record_id = iter.next().ok_or(SqlError::ColumnDoesNotExist)?;
                let value = iter.next().ok_or(SqlError::ColumnDoesNotExist)?;

                Ok(ScalarListElement {
                    record_id: GraphqlId::try_from(record_id)?,
                    value,
                })
            })
            .collect::<connector_interface::Result<Vec<ScalarListElement>>>()?;

        let mut list_values = Vec::new();

        for (record_id, elements) in &results.into_iter().group_by(|ele| ele.record_id.clone()) {
            let values = ScalarListValues {
                record_id,
                values: elements.into_iter().map(|e| e.value).collect(),
            };
            list_values.push(values);
        }

        Ok(list_values)
    }

    fn count_by_model(
        &mut self,
        model: ModelRef,
        query_arguments: QueryArguments,
    ) -> connector_interface::Result<usize> {
        let query = ReadQueryBuilder::count_by_model(model, query_arguments);
        let result = self.inner.find_int(query)? as usize;

        Ok(result)
    }
}