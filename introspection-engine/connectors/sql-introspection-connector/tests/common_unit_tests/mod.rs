use datamodel::{
    common::ScalarType, dml, Datamodel, DefaultValue as DMLDefault, Field, FieldArity, FieldType, IndexDefinition,
    Model, OnDeleteStrategy, RelationInfo, ValueGenerator,
};
use pretty_assertions::assert_eq;
use prisma_value::PrismaValue;
use quaint::connector::SqlFamily;
use sql_introspection_connector::calculate_datamodel::calculate_datamodel;
use sql_schema_describer::*;

#[test]
fn a_data_model_can_be_generated_from_a_schema() {
    let col_types = &[
        ColumnTypeFamily::Int,
        ColumnTypeFamily::Float,
        ColumnTypeFamily::Boolean,
        ColumnTypeFamily::String,
        ColumnTypeFamily::DateTime,
        ColumnTypeFamily::Binary,
        ColumnTypeFamily::Json,
        ColumnTypeFamily::Uuid,
        ColumnTypeFamily::Geometric,
        ColumnTypeFamily::LogSequenceNumber,
        ColumnTypeFamily::TextSearch,
        ColumnTypeFamily::TransactionId,
    ];

    let ref_data_model = Datamodel {
        models: vec![Model {
            database_name: None,
            name: "Table1".to_string(),
            documentation: Some(
                "The underlying table does not contain a unique identifier and can therefore currently not be handled."
                    .to_string(),
            ),
            is_embedded: false,
            is_generated: false,
            is_commented_out: true,
            indices: vec![],
            id_fields: vec![],
            fields: col_types
                .iter()
                .map(|col_type| {
                    let (field_type, is_commented_out, documentation) = match col_type {
                        ColumnTypeFamily::Boolean => (FieldType::Base(ScalarType::Boolean, None), false, None),
                        ColumnTypeFamily::DateTime => (FieldType::Base(ScalarType::DateTime, None), false, None),
                        ColumnTypeFamily::Float => (FieldType::Base(ScalarType::Float, None), false, None),
                        ColumnTypeFamily::Int => (FieldType::Base(ScalarType::Int, None), false, None),
                        ColumnTypeFamily::String => (FieldType::Base(ScalarType::String, None), false, None),
                        ColumnTypeFamily::Enum(name) => (FieldType::Enum(name.clone()), false, None),
                        ColumnTypeFamily::Uuid => (FieldType::Base(ScalarType::String, None), false, None),
                        ColumnTypeFamily::Json => (FieldType::Base(ScalarType::Json, None), false, None),
                        x => (
                            FieldType::Unsupported(x.to_string()),
                            true,
                            Some("This type is currently not supported.".to_string()),
                        ),
                    };
                    Field {
                        name: col_type.to_string(),
                        arity: FieldArity::Optional,
                        field_type,
                        database_name: None,
                        default_value: None,
                        is_unique: false,
                        is_id: false,
                        documentation,
                        is_generated: false,
                        is_updated_at: false,
                        is_commented_out,
                    }
                })
                .collect(),
        }],
        enums: vec![],
    };

    let schema = SqlSchema {
        tables: vec![Table {
            name: "Table1".to_string(),
            columns: col_types
                .iter()
                .map(|family| Column {
                    name: family.to_string(),
                    tpe: ColumnType::pure(family.to_owned(), ColumnArity::Nullable),
                    default: None,
                    auto_increment: false,
                })
                .collect(),
            indices: vec![],
            primary_key: None,
            foreign_keys: vec![],
        }],
        enums: vec![],
        sequences: vec![],
    };
    let introspection_result = calculate_datamodel(&schema, &SqlFamily::Postgres).expect("calculate data model");

    assert_eq!(introspection_result.datamodel, ref_data_model);
}

#[test]
fn arity_is_preserved_when_generating_data_model_from_a_schema() {
    let ref_data_model = Datamodel {
        models: vec![Model {
            database_name: None,
            name: "Table1".to_string(),
            documentation: None,
            is_embedded: false,
            is_commented_out: false,
            fields: vec![
                Field {
                    name: "optional".to_string(),
                    arity: FieldArity::Optional,
                    field_type: FieldType::Base(ScalarType::Int, None),
                    database_name: None,
                    default_value: None,
                    is_unique: false,
                    is_id: false,
                    documentation: None,
                    is_generated: false,
                    is_updated_at: false,
                    is_commented_out: false,
                },
                Field {
                    name: "required".to_string(),
                    arity: FieldArity::Required,
                    field_type: FieldType::Base(ScalarType::Int, None),
                    database_name: None,
                    default_value: Some(DMLDefault::Expression(ValueGenerator::new_autoincrement())),
                    is_unique: false,
                    is_id: true,
                    documentation: None,
                    is_generated: false,
                    is_updated_at: false,
                    is_commented_out: false,
                },
                Field {
                    name: "list".to_string(),
                    arity: FieldArity::List,
                    field_type: FieldType::Base(ScalarType::Int, None),
                    database_name: None,
                    default_value: None,
                    is_unique: false,
                    is_id: false,
                    documentation: None,
                    is_generated: false,
                    is_updated_at: false,
                    is_commented_out: false,
                },
            ],
            is_generated: false,
            indices: vec![],
            id_fields: vec![],
        }],
        enums: vec![],
    };

    let schema = SqlSchema {
        tables: vec![Table {
            name: "Table1".to_string(),
            columns: vec![
                Column {
                    name: "optional".to_string(),
                    tpe: ColumnType::pure(ColumnTypeFamily::Int, ColumnArity::Nullable),
                    default: None,
                    auto_increment: false,
                },
                Column {
                    name: "required".to_string(),
                    tpe: ColumnType::pure(ColumnTypeFamily::Int, ColumnArity::Required),
                    default: None,
                    auto_increment: true,
                },
                Column {
                    name: "list".to_string(),
                    tpe: ColumnType::pure(ColumnTypeFamily::Int, ColumnArity::List),
                    default: None,
                    auto_increment: false,
                },
            ],
            indices: vec![],
            primary_key: Some(PrimaryKey {
                columns: vec!["required".to_string()],
                sequence: None,
            }),
            foreign_keys: vec![],
        }],
        enums: vec![],
        sequences: vec![],
    };
    let introspection_result = calculate_datamodel(&schema, &SqlFamily::Postgres).expect("calculate data model");

    assert_eq!(introspection_result.datamodel, ref_data_model);
}

#[test]
fn defaults_are_preserved_when_generating_data_model_from_a_schema() {
    let ref_data_model = Datamodel {
        models: vec![Model {
            database_name: None,
            name: "Table1".to_string(),
            documentation: None,
            is_embedded: false,
            is_commented_out: false,
            fields: vec![
                Field {
                    name: "no_default".to_string(),
                    arity: FieldArity::Optional,
                    field_type: FieldType::Base(ScalarType::Int, None),
                    database_name: None,
                    default_value: None,
                    is_unique: false,
                    is_id: false,
                    documentation: None,
                    is_generated: false,
                    is_updated_at: false,
                    is_commented_out: false,
                },
                Field {
                    name: "int_default".to_string(),
                    arity: FieldArity::Optional,
                    field_type: FieldType::Base(ScalarType::Int, None),
                    database_name: None,
                    default_value: Some(dml::DefaultValue::Single(PrismaValue::Int(1))),
                    is_unique: false,
                    is_id: false,
                    documentation: None,
                    is_generated: false,
                    is_updated_at: false,
                    is_commented_out: false,
                },
                Field {
                    name: "bool_default".to_string(),
                    arity: FieldArity::Optional,
                    field_type: FieldType::Base(ScalarType::Boolean, None),
                    database_name: None,
                    default_value: Some(dml::DefaultValue::Single(PrismaValue::Boolean(true))),
                    is_unique: false,
                    is_id: false,
                    documentation: None,
                    is_generated: false,
                    is_updated_at: false,
                    is_commented_out: false,
                },
                Field {
                    name: "float_default".to_string(),
                    arity: FieldArity::Optional,
                    field_type: FieldType::Base(ScalarType::Float, None),
                    database_name: None,
                    default_value: Some(dml::DefaultValue::Single(PrismaValue::Float(1.into()))),
                    is_unique: false,
                    is_id: false,
                    documentation: None,
                    is_generated: false,
                    is_updated_at: false,
                    is_commented_out: false,
                },
                Field {
                    name: "string_default".to_string(),
                    arity: FieldArity::Optional,
                    field_type: FieldType::Base(ScalarType::String, None),
                    database_name: None,
                    default_value: Some(dml::DefaultValue::Single(PrismaValue::String("default".to_string()))),
                    is_unique: false,
                    is_id: false,
                    documentation: None,
                    is_generated: false,
                    is_updated_at: false,
                    is_commented_out: false,
                },
            ],
            is_generated: false,
            indices: vec![IndexDefinition {
                name: Some("unique".into()),
                fields: vec!["no_default".into(), "int_default".into()],
                tpe: dml::IndexType::Unique,
            }],
            id_fields: vec![],
        }],
        enums: vec![],
    };

    let schema = SqlSchema {
        tables: vec![Table {
            name: "Table1".to_string(),
            columns: vec![
                Column {
                    name: "no_default".to_string(),
                    tpe: ColumnType::pure(ColumnTypeFamily::Int, ColumnArity::Nullable),
                    default: None,
                    auto_increment: false,
                },
                Column {
                    name: "int_default".to_string(),
                    tpe: ColumnType::pure(ColumnTypeFamily::Int, ColumnArity::Nullable),
                    default: Some(DefaultValue::VALUE(PrismaValue::Int(1))),
                    auto_increment: false,
                },
                Column {
                    name: "bool_default".to_string(),
                    tpe: ColumnType::pure(ColumnTypeFamily::Boolean, ColumnArity::Nullable),
                    default: Some(DefaultValue::VALUE(PrismaValue::Boolean(true))),
                    auto_increment: false,
                },
                Column {
                    name: "float_default".to_string(),
                    tpe: ColumnType::pure(ColumnTypeFamily::Float, ColumnArity::Nullable),
                    default: Some(DefaultValue::VALUE(PrismaValue::new_float(1.0))),
                    auto_increment: false,
                },
                Column {
                    name: "string_default".to_string(),
                    tpe: ColumnType::pure(ColumnTypeFamily::String, ColumnArity::Nullable),
                    default: Some(DefaultValue::VALUE(PrismaValue::String("default".to_string()))),
                    auto_increment: false,
                },
            ],
            indices: vec![Index {
                name: "unique".to_string(),
                columns: vec!["no_default".into(), "int_default".into()],
                tpe: IndexType::Unique,
            }],
            primary_key: None,
            foreign_keys: vec![],
        }],
        enums: vec![],
        sequences: vec![],
    };
    let introspection_result = calculate_datamodel(&schema, &SqlFamily::Postgres).expect("calculate data model");

    assert_eq!(introspection_result.datamodel, ref_data_model);
}

#[test]
fn primary_key_is_preserved_when_generating_data_model_from_a_schema() {
    let ref_data_model = Datamodel {
        models: vec![
            // Model with auto-incrementing primary key
            Model {
                database_name: None,
                name: "Table1".to_string(),
                documentation: None,
                is_embedded: false,
                is_commented_out: false,
                fields: vec![Field {
                    name: "primary".to_string(),
                    arity: FieldArity::Required,
                    field_type: FieldType::Base(ScalarType::Int, None),
                    database_name: None,
                    default_value: Some(DMLDefault::Expression(ValueGenerator::new_autoincrement())),
                    is_unique: false,
                    is_id: true,
                    documentation: None,
                    is_generated: false,
                    is_updated_at: false,
                    is_commented_out: false,
                }],
                is_generated: false,
                indices: vec![],
                id_fields: vec![],
            },
            // Model with non-auto-incrementing primary key
            Model {
                database_name: None,
                name: "Table2".to_string(),
                documentation: None,
                is_embedded: false,
                is_commented_out: false,
                fields: vec![Field {
                    name: "primary".to_string(),
                    arity: FieldArity::Required,
                    field_type: FieldType::Base(ScalarType::Int, None),
                    database_name: None,
                    default_value: None,
                    is_unique: false,
                    is_id: true,
                    documentation: None,
                    is_generated: false,
                    is_updated_at: false,
                    is_commented_out: false,
                }],
                is_generated: false,
                indices: vec![],
                id_fields: vec![],
            },
            // Model with primary key seeded by sequence
            Model {
                database_name: None,
                name: "Table3".to_string(),
                documentation: None,
                is_embedded: false,
                is_commented_out: false,
                fields: vec![Field {
                    name: "primary".to_string(),
                    arity: FieldArity::Required,
                    field_type: FieldType::Base(ScalarType::Int, None),
                    database_name: None,
                    default_value: Some(DMLDefault::Expression(ValueGenerator::new_autoincrement())),
                    is_unique: false,
                    is_id: true,
                    documentation: None,
                    is_generated: false,
                    is_updated_at: false,
                    is_commented_out: false,
                }],
                is_generated: false,
                indices: vec![],
                id_fields: vec![],
            },
        ],
        enums: vec![],
    };

    let schema = SqlSchema {
        tables: vec![
            Table {
                name: "Table1".to_string(),
                columns: vec![Column {
                    name: "primary".to_string(),
                    tpe: ColumnType {
                        data_type: "integer".to_string(),
                        full_data_type: "integer".to_string(),
                        character_maximum_length: None,
                        family: ColumnTypeFamily::Int,
                        arity: ColumnArity::Required,
                    },
                    default: None,
                    auto_increment: true,
                }],
                indices: vec![],
                primary_key: Some(PrimaryKey {
                    columns: vec!["primary".to_string()],
                    sequence: None,
                }),
                foreign_keys: vec![],
            },
            Table {
                name: "Table2".to_string(),
                columns: vec![Column {
                    name: "primary".to_string(),
                    tpe: ColumnType {
                        data_type: "integer".to_string(),
                        full_data_type: "integer".to_string(),
                        character_maximum_length: None,
                        family: ColumnTypeFamily::Int,
                        arity: ColumnArity::Required,
                    },
                    default: None,
                    auto_increment: false,
                }],
                indices: vec![],
                primary_key: Some(PrimaryKey {
                    columns: vec!["primary".to_string()],
                    sequence: None,
                }),
                foreign_keys: vec![],
            },
            Table {
                name: "Table3".to_string(),
                columns: vec![Column {
                    name: "primary".to_string(),
                    tpe: ColumnType {
                        data_type: "integer".to_string(),
                        full_data_type: "integer".to_string(),
                        character_maximum_length: None,

                        family: ColumnTypeFamily::Int,
                        arity: ColumnArity::Required,
                    },
                    default: None,
                    auto_increment: true,
                }],
                indices: vec![],
                primary_key: Some(PrimaryKey {
                    columns: vec!["primary".to_string()],
                    sequence: Some(Sequence {
                        name: "sequence".to_string(),
                        initial_value: 1,
                        allocation_size: 1,
                    }),
                }),
                foreign_keys: vec![],
            },
        ],
        enums: vec![],
        sequences: vec![],
    };
    let introspection_result = calculate_datamodel(&schema, &SqlFamily::Postgres).expect("calculate data model");

    assert_eq!(introspection_result.datamodel, ref_data_model);
}

#[test]
fn uniqueness_is_preserved_when_generating_data_model_from_a_schema() {
    let ref_data_model = Datamodel {
        models: vec![Model {
            database_name: None,
            name: "Table1".to_string(),
            documentation: None,
            is_embedded: false,
            is_commented_out: false,
            fields: vec![
                Field {
                    name: "non_unique".to_string(),
                    arity: FieldArity::Optional,
                    field_type: FieldType::Base(ScalarType::Int, None),
                    database_name: None,
                    default_value: None,
                    is_unique: false,
                    is_id: false,
                    documentation: None,
                    is_generated: false,
                    is_updated_at: false,
                    is_commented_out: false,
                },
                Field {
                    name: "unique".to_string(),
                    arity: FieldArity::Required,
                    field_type: FieldType::Base(ScalarType::Int, None),
                    database_name: None,
                    default_value: None,
                    is_unique: true,
                    is_id: false,
                    documentation: None,
                    is_generated: false,
                    is_updated_at: false,
                    is_commented_out: false,
                },
            ],
            is_generated: false,
            indices: vec![],
            id_fields: vec![],
        }],
        enums: vec![],
    };

    let schema = SqlSchema {
        tables: vec![Table {
            name: "Table1".to_string(),
            columns: vec![
                Column {
                    name: "non_unique".to_string(),
                    tpe: ColumnType::pure(ColumnTypeFamily::Int, ColumnArity::Nullable),
                    default: None,
                    auto_increment: false,
                },
                Column {
                    name: "unique".to_string(),
                    tpe: ColumnType::pure(ColumnTypeFamily::Int, ColumnArity::Required),
                    default: None,
                    auto_increment: false,
                },
            ],
            indices: vec![Index {
                name: "unique".to_string(),
                columns: vec!["unique".to_string()],
                tpe: IndexType::Unique,
            }],
            primary_key: None,
            foreign_keys: vec![],
        }],
        enums: vec![],
        sequences: vec![],
    };
    let introspection_result = calculate_datamodel(&schema, &SqlFamily::Postgres).expect("calculate data model");

    assert_eq!(introspection_result.datamodel, ref_data_model);
}

#[test]
fn compound_foreign_keys_are_preserved_when_generating_data_model_from_a_schema() {
    let expected_data_model = Datamodel {
        models: vec![
            Model {
                database_name: None,
                name: "City".to_string(),
                documentation: None,
                is_embedded: false,
                is_commented_out: false,
                fields: vec![
                    Field {
                        name: "id".to_string(),
                        arity: FieldArity::Required,
                        field_type: FieldType::Base(ScalarType::Int, None),
                        database_name: None,
                        default_value: Some(DMLDefault::Expression(ValueGenerator::new_autoincrement())),
                        is_unique: false,
                        is_id: true,
                        documentation: None,
                        is_generated: false,
                        is_updated_at: false,
                        is_commented_out: false,
                    },
                    Field {
                        name: "name".to_string(),
                        arity: FieldArity::Required,
                        field_type: FieldType::Base(ScalarType::String, None),
                        database_name: None,
                        default_value: None,
                        is_unique: false,
                        is_id: false,
                        documentation: None,
                        is_generated: false,
                        is_updated_at: false,
                        is_commented_out: false,
                    },
                    Field {
                        name: "User".to_string(),
                        field_type: FieldType::Relation(RelationInfo {
                            to: "User".to_string(),
                            fields: vec![],
                            to_fields: vec![],
                            name: "CityToUser".to_string(),
                            on_delete: OnDeleteStrategy::None,
                        }),
                        arity: FieldArity::List,
                        database_name: None,
                        default_value: None,
                        is_unique: false,
                        is_id: false,
                        documentation: None,
                        is_generated: false,
                        is_updated_at: false,
                        is_commented_out: false,
                    },
                ],
                is_generated: false,
                indices: vec![],
                id_fields: vec![],
            },
            Model {
                database_name: None,
                name: "User".to_string(),
                documentation: None,
                is_embedded: false,
                is_commented_out: false,
                fields: vec![
                    Field {
                        name: "id".to_string(),
                        arity: FieldArity::Required,
                        field_type: FieldType::Base(ScalarType::Int, None),
                        database_name: None,
                        default_value: Some(DMLDefault::Expression(ValueGenerator::new_autoincrement())),
                        is_unique: false,
                        is_id: true,
                        documentation: None,
                        is_generated: false,
                        is_updated_at: false,
                        is_commented_out: false,
                    },
                    Field {
                        name: "city_id".to_string(),
                        arity: FieldArity::Required,
                        field_type: FieldType::Base(ScalarType::Int, None),
                        database_name: Some("city-id".to_string()),
                        default_value: None,
                        is_unique: false,
                        is_id: false,
                        documentation: None,
                        is_generated: false,
                        is_updated_at: false,
                        is_commented_out: false,
                    },
                    Field {
                        name: "city_name".to_string(),
                        field_type: FieldType::Base(ScalarType::String, None),
                        arity: FieldArity::Required,
                        database_name: Some("city-name".to_string()),
                        default_value: None,
                        is_unique: false,
                        is_id: false,
                        documentation: None,
                        is_generated: false,
                        is_updated_at: false,
                        is_commented_out: false,
                    },
                    Field {
                        name: "City".to_string(),
                        arity: FieldArity::Required,
                        field_type: FieldType::Relation(RelationInfo {
                            name: "CityToUser".to_string(),
                            to: "City".to_string(),
                            fields: vec!["city_id".to_string(), "city_name".to_string()],
                            to_fields: vec!["id".to_string(), "name".to_string()],
                            on_delete: OnDeleteStrategy::None,
                        }),
                        database_name: None,
                        default_value: None,
                        is_unique: false,
                        is_id: false,
                        documentation: None,
                        is_generated: false,
                        is_updated_at: false,
                        is_commented_out: false,
                    },
                ],
                is_generated: false,
                indices: vec![],
                id_fields: vec![],
            },
        ],
        enums: vec![],
    };

    let schema = SqlSchema {
        tables: vec![
            Table {
                name: "City".to_string(),
                columns: vec![
                    Column {
                        name: "id".to_string(),
                        tpe: ColumnType {
                            data_type: "integer".to_string(),
                            full_data_type: "integer".to_string(),
                            character_maximum_length: None,

                            family: ColumnTypeFamily::Int,
                            arity: ColumnArity::Required,
                        },
                        default: None,
                        auto_increment: true,
                    },
                    Column {
                        name: "name".to_string(),
                        tpe: ColumnType {
                            data_type: "text".to_string(),
                            full_data_type: "text".to_string(),
                            character_maximum_length: None,

                            family: ColumnTypeFamily::String,
                            arity: ColumnArity::Required,
                        },
                        default: None,
                        auto_increment: false,
                    },
                ],
                indices: vec![],
                primary_key: Some(PrimaryKey {
                    columns: vec!["id".to_string()],
                    sequence: None,
                }),
                foreign_keys: vec![],
            },
            Table {
                name: "User".to_string(),
                columns: vec![
                    Column {
                        name: "id".to_string(),
                        tpe: ColumnType {
                            data_type: "integer".to_string(),
                            full_data_type: "integer".to_string(),
                            character_maximum_length: None,

                            family: ColumnTypeFamily::Int,
                            arity: ColumnArity::Required,
                        },
                        default: None,
                        auto_increment: true,
                    },
                    Column {
                        name: "city-id".to_string(),
                        tpe: ColumnType {
                            data_type: "integer".to_string(),
                            full_data_type: "integer".to_string(),
                            character_maximum_length: None,

                            family: ColumnTypeFamily::Int,
                            arity: ColumnArity::Required,
                        },
                        default: None,
                        auto_increment: false,
                    },
                    Column {
                        name: "city-name".to_string(),
                        tpe: ColumnType {
                            data_type: "text".to_string(),
                            full_data_type: "text".to_string(),
                            character_maximum_length: None,

                            family: ColumnTypeFamily::String,
                            arity: ColumnArity::Required,
                        },
                        default: None,
                        auto_increment: false,
                    },
                ],
                indices: vec![],
                primary_key: Some(PrimaryKey {
                    columns: vec!["id".to_string()],
                    sequence: None,
                }),
                foreign_keys: vec![ForeignKey {
                    // what does this mean? the from columns are not targeting a specific to column?
                    constraint_name: None,
                    columns: vec!["city-id".to_string(), "city-name".to_string()],
                    referenced_table: "City".to_string(),
                    on_delete_action: ForeignKeyAction::NoAction,
                    referenced_columns: vec!["id".to_string(), "name".to_string()],
                }],
            },
        ],
        enums: vec![],
        sequences: vec![],
    };
    let introspection_result = calculate_datamodel(&schema, &SqlFamily::Postgres).expect("calculate data model");

    assert_eq!(introspection_result.datamodel, expected_data_model);
}

#[test]
fn multi_field_uniques_are_preserved_when_generating_data_model_from_a_schema() {
    let ref_data_model = Datamodel {
        models: vec![Model {
            database_name: None,
            name: "User".to_string(),
            documentation: None,
            is_embedded: false,
            is_commented_out: false,
            fields: vec![
                Field {
                    name: "id".to_string(),
                    arity: FieldArity::Required,
                    field_type: FieldType::Base(ScalarType::Int, None),
                    database_name: None,
                    default_value: Some(DMLDefault::Expression(ValueGenerator::new_autoincrement())),
                    is_unique: false,
                    is_id: true,
                    documentation: None,
                    is_generated: false,
                    is_updated_at: false,
                    is_commented_out: false,
                },
                Field {
                    name: "name".to_string(),
                    arity: FieldArity::Required,
                    field_type: FieldType::Base(ScalarType::String, None),
                    database_name: None,
                    default_value: None,
                    is_unique: false,
                    is_id: false,
                    documentation: None,
                    is_generated: false,
                    is_updated_at: false,
                    is_commented_out: false,
                },
                Field {
                    name: "lastname".to_string(),
                    arity: FieldArity::Required,
                    field_type: FieldType::Base(ScalarType::String, None),
                    database_name: None,
                    default_value: None,
                    is_unique: false,
                    is_id: false,
                    documentation: None,
                    is_generated: false,
                    is_updated_at: false,
                    is_commented_out: false,
                },
            ],
            is_generated: false,
            indices: vec![datamodel::dml::IndexDefinition {
                name: Some("name_last_name_unique".to_string()),
                fields: vec!["name".to_string(), "lastname".to_string()],
                tpe: datamodel::dml::IndexType::Unique,
            }],
            id_fields: vec![],
        }],
        enums: vec![],
    };

    let schema = SqlSchema {
        tables: vec![Table {
            name: "User".to_string(),
            columns: vec![
                Column {
                    name: "id".to_string(),
                    tpe: ColumnType {
                        data_type: "integer".to_string(),
                        full_data_type: "integer".to_string(),
                        character_maximum_length: None,

                        family: ColumnTypeFamily::Int,
                        arity: ColumnArity::Required,
                    },
                    default: None,
                    auto_increment: true,
                },
                Column {
                    name: "name".to_string(),
                    tpe: ColumnType {
                        data_type: "text".to_string(),
                        full_data_type: "text".to_string(),
                        character_maximum_length: None,

                        family: ColumnTypeFamily::String,
                        arity: ColumnArity::Required,
                    },
                    default: None,
                    auto_increment: false,
                },
                Column {
                    name: "lastname".to_string(),
                    tpe: ColumnType {
                        data_type: "text".to_string(),
                        full_data_type: "text".to_string(),
                        character_maximum_length: None,

                        family: ColumnTypeFamily::String,
                        arity: ColumnArity::Required,
                    },
                    default: None,
                    auto_increment: false,
                },
            ],
            indices: vec![Index {
                name: "name_last_name_unique".to_string(),
                columns: vec!["name".to_string(), "lastname".to_string()],
                tpe: IndexType::Unique,
            }],
            primary_key: Some(PrimaryKey {
                columns: vec!["id".to_string()],
                sequence: None,
            }),
            foreign_keys: vec![],
        }],
        enums: vec![],
        sequences: vec![],
    };
    let introspection_result = calculate_datamodel(&schema, &SqlFamily::Postgres).expect("calculate data model");

    assert_eq!(introspection_result.datamodel, ref_data_model);
}

#[test]
fn foreign_keys_are_preserved_when_generating_data_model_from_a_schema() {
    let ref_data_model = Datamodel {
        models: vec![
            Model {
                database_name: None,
                name: "City".to_string(),
                documentation: None,
                is_embedded: false,
                is_commented_out: false,
                fields: vec![
                    Field {
                        name: "id".to_string(),
                        arity: FieldArity::Required,
                        field_type: FieldType::Base(ScalarType::Int, None),
                        database_name: None,
                        default_value: Some(DMLDefault::Expression(ValueGenerator::new_autoincrement())),
                        is_unique: false,
                        is_id: true,
                        documentation: None,
                        is_generated: false,
                        is_updated_at: false,
                        is_commented_out: false,
                    },
                    Field {
                        name: "name".to_string(),
                        arity: FieldArity::Required,
                        field_type: FieldType::Base(ScalarType::String, None),
                        database_name: None,
                        default_value: None,
                        is_unique: false,
                        is_id: false,
                        documentation: None,
                        is_generated: false,
                        is_updated_at: false,
                        is_commented_out: false,
                    },
                    Field {
                        name: "User".to_string(),
                        arity: FieldArity::List,
                        field_type: FieldType::Relation(RelationInfo {
                            to: "User".to_string(),
                            fields: vec![],
                            to_fields: vec![],
                            name: "CityToUser".to_string(),
                            on_delete: OnDeleteStrategy::None,
                        }),
                        database_name: None,
                        default_value: None,
                        is_unique: false,
                        is_id: false,
                        documentation: None,
                        is_generated: false,
                        is_updated_at: false,
                        is_commented_out: false,
                    },
                ],
                is_generated: false,
                indices: vec![],
                id_fields: vec![],
            },
            Model {
                database_name: None,
                name: "User".to_string(),
                documentation: None,
                is_embedded: false,
                is_commented_out: false,
                fields: vec![
                    Field {
                        name: "id".to_string(),
                        arity: FieldArity::Required,
                        field_type: FieldType::Base(ScalarType::Int, None),
                        database_name: None,
                        default_value: Some(DMLDefault::Expression(ValueGenerator::new_autoincrement())),
                        is_unique: false,
                        is_id: true,
                        documentation: None,
                        is_generated: false,
                        is_updated_at: false,
                        is_commented_out: false,
                    },
                    Field {
                        name: "city_id".to_string(),
                        field_type: FieldType::Base(ScalarType::Int, None),
                        arity: FieldArity::Required,
                        database_name: None,
                        default_value: None,
                        is_unique: false,
                        is_id: false,
                        documentation: None,
                        is_generated: false,
                        is_updated_at: false,
                        is_commented_out: false,
                    },
                    Field {
                        name: "City".to_string(),
                        arity: FieldArity::Required,
                        field_type: FieldType::Relation(RelationInfo {
                            name: "CityToUser".to_string(),
                            to: "City".to_string(),
                            fields: vec!["city_id".to_string()],
                            to_fields: vec!["id".to_string()],
                            on_delete: OnDeleteStrategy::None,
                        }),
                        database_name: None,
                        default_value: None,
                        is_unique: false,
                        is_id: false,
                        documentation: None,
                        is_generated: false,
                        is_updated_at: false,
                        is_commented_out: false,
                    },
                ],
                is_generated: false,
                indices: vec![],
                id_fields: vec![],
            },
        ],
        enums: vec![],
    };

    let schema = SqlSchema {
        tables: vec![
            Table {
                name: "City".to_string(),
                columns: vec![
                    Column {
                        name: "id".to_string(),
                        tpe: ColumnType {
                            data_type: "integer".to_string(),
                            full_data_type: "integer".to_string(),
                            character_maximum_length: None,

                            family: ColumnTypeFamily::Int,
                            arity: ColumnArity::Required,
                        },
                        default: None,
                        auto_increment: true,
                    },
                    Column {
                        name: "name".to_string(),
                        tpe: ColumnType {
                            data_type: "text".to_string(),
                            full_data_type: "text".to_string(),
                            character_maximum_length: None,

                            family: ColumnTypeFamily::String,
                            arity: ColumnArity::Required,
                        },
                        default: None,
                        auto_increment: false,
                    },
                ],
                indices: vec![],
                primary_key: Some(PrimaryKey {
                    columns: vec!["id".to_string()],
                    sequence: None,
                }),
                foreign_keys: vec![],
            },
            Table {
                name: "User".to_string(),
                columns: vec![
                    Column {
                        name: "id".to_string(),
                        tpe: ColumnType {
                            data_type: "integer".to_string(),
                            full_data_type: "integer".to_string(),
                            character_maximum_length: None,

                            family: ColumnTypeFamily::Int,
                            arity: ColumnArity::Required,
                        },
                        default: None,
                        auto_increment: true,
                    },
                    Column {
                        name: "city_id".to_string(),
                        tpe: ColumnType {
                            data_type: "integer".to_string(),
                            full_data_type: "integer".to_string(),
                            character_maximum_length: None,

                            family: ColumnTypeFamily::Int,
                            arity: ColumnArity::Required,
                        },
                        default: None,
                        auto_increment: false,
                    },
                ],
                indices: vec![],
                primary_key: Some(PrimaryKey {
                    columns: vec!["id".to_string()],
                    sequence: None,
                }),
                foreign_keys: vec![ForeignKey {
                    constraint_name: None,
                    columns: vec!["city_id".to_string()],
                    referenced_table: "City".to_string(),
                    on_delete_action: ForeignKeyAction::NoAction,
                    referenced_columns: vec!["id".to_string()],
                }],
            },
        ],
        enums: vec![],
        sequences: vec![],
    };
    let introspection_result = calculate_datamodel(&schema, &SqlFamily::Postgres).expect("calculate data model");

    assert_eq!(introspection_result.datamodel, ref_data_model);
}

#[test]
fn enums_are_preserved_when_generating_data_model_from_a_schema() {
    let ref_data_model = Datamodel {
        models: vec![],
        enums: vec![dml::Enum {
            name: "Enum".to_string(),
            database_name: None,
            documentation: None,
            values: vec![
                datamodel::dml::EnumValue {
                    name: "a".to_string(),
                    documentation: None,
                    database_name: None,
                    commented_out: false,
                },
                datamodel::dml::EnumValue {
                    name: "b".to_string(),
                    documentation: None,
                    database_name: None,
                    commented_out: false,
                },
            ],
        }],
    };

    let enum_values = vec!["a".to_string(), "b".to_string()];
    let schema = SqlSchema {
        tables: vec![],
        enums: vec![Enum {
            name: "Enum".to_string(),
            values: enum_values,
        }],
        sequences: vec![],
    };
    let introspection_result = calculate_datamodel(&schema, &SqlFamily::Postgres).expect("calculate data model");

    assert_eq!(introspection_result.datamodel, ref_data_model);
}
