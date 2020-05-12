mod existing_data;

use migration_connector::MigrationWarning;
use migration_engine_tests::sql::*;
use pretty_assertions::assert_eq;
use prisma_value::PrismaValue;
use quaint::ast::*;
use std::borrow::Cow;

#[test_each_connector]
async fn dropping_a_table_with_rows_should_warn(api: &TestApi) {
    let dm = r#"
        model Test {
            id String @id @default(cuid())
        }
    "#;
    let original_database_schema = api.infer_and_apply(&dm).await.sql_schema;

    let conn = api.database();
    let insert = Insert::single_into((api.schema_name(), "Test")).value("id", "test");

    conn.query(insert.into()).await.unwrap();

    let dm = "";

    let InferAndApplyOutput {
        migration_output,
        sql_schema: final_database_schema,
    } = api.infer_and_apply(&dm).await;

    // The schema should not change because the migration should not run if there are warnings
    // and the force flag isn't passed.
    assert_eq!(original_database_schema, final_database_schema);

    assert_eq!(
        migration_output.warnings,
        &[MigrationWarning {
            description: "You are about to drop the table `Test`, which is not empty (1 rows).".into()
        }]
    );
}

#[test_each_connector]
async fn dropping_a_column_with_non_null_values_should_warn(api: &TestApi) {
    let dm = r#"
            model Test {
                id String @id @default(cuid())
                puppiesCount Int?
            }
        "#;

    let original_database_schema = api.infer_and_apply(&dm).await.sql_schema;

    let insert = Insert::multi_into((api.schema_name(), "Test"), &["id", "puppiesCount"])
        .values(("a", 7))
        .values(("b", 8));

    api.database().query(insert.into()).await.unwrap();

    // Drop the `favouriteAnimal` column.
    let dm = r#"
            model Test {
                id String @id @default(cuid())
            }
        "#;

    let InferAndApplyOutput {
        migration_output,
        sql_schema: final_database_schema,
    } = api.infer_and_apply(&dm).await;

    // The schema should not change because the migration should not run if there are warnings
    // and the force flag isn't passed.
    assert_eq!(original_database_schema, final_database_schema);

    assert_eq!(
            migration_output.warnings,
            &[MigrationWarning {
                description: "You are about to drop the column `puppiesCount` on the `Test` table, which still contains 2 non-null values.".to_owned(),
            }]
        );
}

#[test_each_connector]
async fn altering_a_column_without_non_null_values_should_warn(api: &TestApi) -> TestResult {
    let dm = r#"
        model Test {
            id String @id @default(cuid())
            puppiesCount Int?
        }
    "#;

    let original_database_schema = api.infer_and_apply(&dm).await.sql_schema;

    let insert = Insert::multi_into((api.schema_name(), "Test"), &["id"])
        .values(("a",))
        .values(("b",));

    api.database().query(insert.into()).await.unwrap();

    let dm2 = r#"
        model Test {
            id String @id @default(cuid())
            puppiesCount Float?
        }
    "#;

    let result = api.infer_apply(&dm2).send().await?.into_inner();

    api.assert_schema().await?.assert_equals(&original_database_schema)?;

    // This one should warn because it would fail on MySQL. TODO: improve the message.

    assert_eq!(
        result.warnings,
        &[MigrationWarning {
            description:
                "You are about to alter the column `puppiesCount` on the `Test` table, which still contains 2 values. \
                 The data in that column may be lost."
                    .to_owned()
        }]
    );

    Ok(())
}

#[test_each_connector]
async fn altering_a_column_with_non_null_values_should_warn(api: &TestApi) -> TestResult {
    let dm = r#"
        model Test {
            id String @id @default(cuid())
            age Int?
        }
    "#;

    api.infer_apply(&dm).send().await?.assert_green()?;
    let original_database_schema = api.describe_database().await?;

    let insert = Insert::multi_into((api.schema_name(), "Test"), vec!["id", "age"])
        .values(("a", 12))
        .values(("b", 22));

    api.database().query(insert.into()).await.unwrap();

    let dm2 = r#"
        model Test {
            id String @id @default(cuid())
            age Float?
        }
    "#;

    let migration_output = api.infer_apply(&dm2).send().await?.into_inner();
    // The schema should not change because the migration should not run if there are warnings
    // and the force flag isn't passed.
    api.assert_schema().await?.assert_equals(&original_database_schema)?;

    assert_eq!(
        migration_output.warnings,
        &[MigrationWarning {
            description: "You are about to alter the column `age` on the `Test` table, which still contains 2 values. \
                 The data in that column may be lost."
                .to_owned()
        }]
    );

    let data = api.dump_table("Test").await?;
    assert_eq!(data.len(), 2);

    Ok(())
}

#[test_each_connector(log = "debug,sql_schema_describer=info")]
async fn column_defaults_can_safely_be_changed(api: &TestApi) -> TestResult {
    let combinations = &[
        ("Meow", Some(PrismaValue::String("Cats".to_string())), None),
        ("Freedom", None, Some(PrismaValue::String("Braveheart".to_string()))),
        (
            "OutstandingMovies",
            Some(PrismaValue::String("Cats".to_string())),
            Some(PrismaValue::String("Braveheart".to_string())),
        ),
    ];

    for (model_name, first_default, second_default) in combinations {
        let span = tracing::info_span!("Combination", model_name, ?first_default, ?second_default);
        let _combination_scope = span.enter();
        tracing::info!("Testing new combination");

        // Set up the initial schema
        {
            let dm1 = format!(
                r#"
                    model {} {{
                        id String @id
                        name String? {}
                    }}
                "#,
                model_name,
                first_default
                    .as_ref()
                    .map(|default| format!("@default(\"{}\")", default))
                    .unwrap_or_else(String::new)
            );

            api.infer_apply(&dm1).force(Some(true)).send().await?;

            api.assert_schema().await?.assert_table(model_name, |table| {
                table.assert_column("name", |column| {
                    if let Some(first_default) = first_default.as_ref() {
                        column.assert_default_value(first_default)
                    } else {
                        column.assert_has_no_default()
                    }
                })
            })?;
        }

        // Insert data
        {
            let insert_span = tracing::info_span!("Data insertion");
            let _insert_scope = insert_span.enter();

            let query = Insert::single_into(api.render_table_name(model_name)).value("id", "abc");

            api.database().query(query.into()).await?;

            let query = Insert::single_into(api.render_table_name(model_name))
                .value("id", "def")
                .value("name", "Waterworld");

            api.database().query(query.into()).await?;

            let data = api.dump_table(model_name).await?;
            let names: Vec<PrismaValue> = data
                .into_iter()
                .filter_map(|row| {
                    row.get("name").map(|val| {
                        val.to_string()
                            .map(|val| PrismaValue::String(val))
                            .unwrap_or(PrismaValue::Null)
                    })
                })
                .collect();

            assert_eq!(
                &[
                    first_default.as_ref().cloned().unwrap_or(PrismaValue::Null),
                    PrismaValue::String("Waterworld".to_string())
                ],
                names.as_slice()
            );
        }

        // Migrate
        {
            let dm2 = format!(
                r#"
                    model {} {{
                        id String @id
                        name String? {}
                    }}
                "#,
                model_name,
                second_default
                    .as_ref()
                    .map(|default| format!(r#"@default("{}")"#, default))
                    .unwrap_or_else(String::new)
            );

            let response = api.infer_apply(&dm2).force(Some(true)).send().await?;

            if api.is_mysql() {
                // On MySQL we have warnings because MODIFY needs to restate the type of the column, and that may be wrong.
                response.assert_executable()?;
            } else {
                response.assert_green()?;
            }
        }

        // Check that the data is still there
        {
            let data = api.dump_table(model_name).await?;
            let names: Vec<PrismaValue> = data
                .into_iter()
                .filter_map(|row| {
                    row.get("name").map(|val| {
                        val.to_string()
                            .map(|val| PrismaValue::String(val))
                            .unwrap_or(PrismaValue::Null)
                    })
                })
                .collect();
            assert_eq!(
                &[
                    first_default.as_ref().cloned().unwrap_or(PrismaValue::Null),
                    PrismaValue::String("Waterworld".to_string())
                ],
                names.as_slice()
            );

            api.assert_schema().await?.assert_table(model_name, |table| {
                table.assert_column("name", |column| {
                    if let Some(second_default) = second_default.as_ref() {
                        column.assert_default_value(second_default)
                    } else {
                        column.assert_has_no_default()
                    }
                })
            })?;
        }
    }

    Ok(())
}

#[test_each_connector(log = "debug")]
async fn set_default_current_timestamp_on_existing_column_works(api: &TestApi) -> TestResult {
    let dm1 = r#"
        model User {
            id Int @id
            created_at DateTime
        }
    "#;

    api.infer_apply(dm1).send().await?.assert_green()?;

    let dm2 = r#"
        model User {
            id Int @id
            created_at DateTime @default(now())
        }
    "#;

    api.infer_apply(dm2).send().await?.assert_green()?;

    Ok(())
}

#[test_each_connector]
async fn changing_a_column_from_required_to_optional_should_work(api: &TestApi) -> TestResult {
    let dm = r#"
        model Test {
            id String @id @default(cuid())
            age Int @default(30)
        }
    "#;

    api.infer_apply(&dm).send().await?.assert_green()?;
    let original_database_schema = api.describe_database().await?;

    let insert = Insert::multi_into((api.schema_name(), "Test"), &["id", "age"])
        .values(("a", 12))
        .values(("b", 22));

    api.database().query(insert.into()).await.unwrap();

    let dm2 = r#"
        model Test {
            id String @id @default(cuid())
            age Int? @default(30)
        }
    "#;

    let migration_output = api.infer_apply(&dm2).send().await?.into_inner();

    if api.is_mysql() {
        anyhow::ensure!(
            migration_output.warnings.len() == 1,
            "Migration warnings should have one warning on mysql. Got {:#?}",
            migration_output.warnings
        );

        assert_eq!(
            migration_output.warnings.get(0).unwrap().description,
            "You are about to alter the column `age` on the `Test` table, which still contains 2 values. The data in that column may be lost.",
        );

        api.assert_schema().await?.assert_equals(&original_database_schema)?;
    } else {
        // On other databases, the migration should be successful.
        anyhow::ensure!(
            migration_output.warnings.is_empty(),
            "Migration warnings should be empty. Got {:#?}",
            migration_output.warnings
        );

        api.assert_schema().await?.assert_ne(&original_database_schema)?;
    }

    // Check that no data was lost.
    {
        let data = api.dump_table("Test").await?;
        assert_eq!(data.len(), 2);
        let ages: Vec<i64> = data
            .into_iter()
            .map(|row| row.get("age").unwrap().as_i64().unwrap())
            .collect();

        assert_eq!(ages, &[12, 22]);
    }

    Ok(())
}

#[test_each_connector]
async fn changing_a_column_from_optional_to_required_must_warn(api: &TestApi) -> TestResult {
    let dm = r#"
        model Test {
            id String @id @default(cuid())
            age Int?
        }
    "#;

    api.infer_apply(&dm).send().await?.assert_green()?;
    let original_database_schema = api.describe_database().await?;

    let insert = Insert::multi_into((api.schema_name(), "Test"), &["id", "age"])
        .values(("a", 12))
        .values(("b", 22));

    api.database().query(insert.into()).await.unwrap();

    let dm2 = r#"
        model Test {
            id String @id @default(cuid())
            age Int @default(30)
        }
    "#;

    let migration_output = api.infer_apply(&dm2).send().await?.into_inner();

    // The schema should not change because the migration should not run if there are warnings
    // and the force flag isn't passed.
    api.assert_schema().await?.assert_equals(&original_database_schema)?;

    assert_eq!(
        migration_output.warnings,
        &[MigrationWarning {
            description: "You are about to alter the column `age` on the `Test` table, which still contains 2 values. \
                 The data in that column may be lost."
                .to_owned()
        }]
    );

    // Check that no data was lost.
    {
        let data = api.dump_table("Test").await?;
        assert_eq!(data.len(), 2);
        let ages: Vec<i64> = data
            .into_iter()
            .map(|row| row.get("age").unwrap().as_i64().unwrap())
            .collect();

        assert_eq!(ages, &[12, 22]);
    }

    Ok(())
}

#[test_each_connector(tags("sql"))]
async fn dropping_a_table_referenced_by_foreign_keys_must_work(api: &TestApi) -> TestResult {
    use quaint::ast::*;

    let dm1 = r#"
        model Category {
            id Int @id
            name String
        }

        model Recipe {
            id Int @id
            categoryId Int
            category Category @relation(fields: [categoryId], references: [id])
        }
    "#;

    api.infer_apply(&dm1).send().await?.assert_green()?;

    api.assert_schema()
        .await?
        .assert_table("Category", |table| table.assert_columns_count(2))?
        .assert_table("Recipe", |table| {
            table.assert_fk_on_columns(&["categoryId"], |fk| fk.assert_references("Category", &["id"]))
        })?;

    let id: i32 = 1;

    let insert = Insert::single_into(api.render_table_name("Category"))
        .value("name", "desserts")
        .value("id", id);
    api.database().query(insert.into()).await?;

    let insert = Insert::single_into(api.render_table_name("Recipe"))
        .value("categoryId", id)
        .value("id", id);
    api.database().query(insert.into()).await?;

    let dm2 = r#"
        model Recipe {
            id Int @id
        }
    "#;

    api.infer_apply(dm2).force(Some(true)).send().await?.into_inner();
    let sql_schema = api.describe_database().await.unwrap();

    assert!(sql_schema.table("Category").is_err());
    assert!(sql_schema.table_bang("Recipe").foreign_keys.is_empty());

    Ok(())
}

#[test_each_connector]
async fn string_columns_do_not_get_arbitrarily_migrated(api: &TestApi) -> TestResult {
    use quaint::ast::*;

    let dm1 = r#"
        model User {
            id           String  @default(cuid()) @id
            name         String?
            email        String  @unique
            kindle_email String? @unique
        }
    "#;

    api.infer_apply(dm1).send().await?.assert_green()?;

    let insert = Insert::single_into(api.render_table_name("User"))
        .value("id", "the-id")
        .value("name", "George")
        .value("email", "george@prisma.io")
        .value("kindle_email", "george+kindle@prisma.io");

    api.database().query(insert.into()).await?;

    let dm2 = r#"
        model User {
            id           String  @default(cuid()) @id
            name         String?
            email        String  @unique
            kindle_email String? @unique
            count        Int     @default(0)
        }
    "#;

    let output = api.infer_apply(dm2).send().await?.assert_green()?.into_inner();

    assert!(output.warnings.is_empty());

    // Check that the string values are still there.
    let select = Select::from_table(api.render_table_name("User"))
        .column("name")
        .column("kindle_email")
        .column("email");

    let counts = api.database().query(select.into()).await?;

    let row = counts.get(0).unwrap();

    assert_eq!(row.get("name").unwrap().as_str().unwrap(), "George");
    assert_eq!(
        row.get("kindle_email").unwrap().as_str().unwrap(),
        "george+kindle@prisma.io"
    );
    assert_eq!(row.get("email").unwrap().as_str().unwrap(), "george@prisma.io");

    Ok(())
}

#[test_each_connector]
async fn altering_the_type_of_a_column_in_an_empty_table_should_not_warn(api: &TestApi) -> TestResult {
    let dm1 = r#"
        model User {
            id String @id @default(cuid())
            name String
            dogs Int
        }
    "#;

    api.infer_apply(dm1).send().await?.assert_green()?;

    let dm2 = r#"
        model User {
            id String @id @default(cuid())
            name String
            dogs String
        }
    "#;

    let response = api.infer_apply(dm2).send().await?.assert_green()?.into_inner();

    assert!(response.warnings.is_empty());

    api.assert_schema()
        .await?
        .assert_table("User", |table| {
            table.assert_column("dogs", |col| col.assert_type_is_string()?.assert_is_required())
        })
        .map(drop)
}

#[test_each_connector]
async fn making_a_column_required_in_an_empty_table_should_not_warn(api: &TestApi) -> TestResult {
    let dm1 = r#"
        model User {
            id String @id @default(cuid())
            name String
            dogs Int?
        }
    "#;

    api.infer_apply(dm1).send().await?.assert_green()?;

    let dm2 = r#"
        model User {
            id String @id @default(cuid())
            name String
            dogs Int
        }
    "#;

    let response = api.infer_apply(dm2).send().await?.assert_green()?.into_inner();

    assert!(response.warnings.is_empty());

    api.assert_schema()
        .await?
        .assert_table("User", |table| {
            table.assert_column("dogs", |col| col.assert_type_is_int()?.assert_is_required())
        })
        .map(drop)
}

#[test_each_connector]
async fn altering_the_type_of_a_column_in_a_non_empty_table_always_warns(api: &TestApi) -> TestResult {
    let dm1 = r#"
        model User {
            id String @id @default(cuid())
            name String
            dogs Int
        }
    "#;

    api.infer_apply(dm1).send().await?.assert_green()?;

    let insert = quaint::ast::Insert::single_into(api.render_table_name("User"))
        .value("id", "abc")
        .value("name", "Shinzo")
        .value("dogs", 7);

    api.database().query(insert.into()).await?;

    let dm2 = r#"
        model User {
            id String @id @default(cuid())
            name String
            dogs String
        }
    "#;

    let response = api.infer_apply(dm2).send().await?.into_inner();

    assert_eq!(
        response.warnings,
        &[MigrationWarning {
            // TODO: the message should say that altering the type of a column is not guaranteed to preserve the data, but the database is going to do its best.
            // Also think about timeouts.
            description: "You are about to alter the column `dogs` on the `User` table, which still contains 1 values. The data in that column may be lost.".to_owned()
        }]
    );

    let rows = api.select("User").column("dogs").send_debug().await?;
    assert_eq!(rows, &[["Integer(7)"]]);

    api.assert_schema().await?.assert_table("User", |table| {
        table.assert_column("dogs", |col| col.assert_type_is_int()?.assert_is_required())
    })?;

    Ok(())
}

#[test_each_connector(ignore("mysql"))]
async fn migrating_a_required_column_from_int_to_string_should_warn_and_cast(api: &TestApi) -> TestResult {
    let dm1 = r#"
        model Test {
            id String @id
            serialNumber Int
        }
    "#;

    api.infer_apply(dm1).send().await?.assert_green()?;

    api.insert("Test")
        .value("id", "abcd")
        .value("serialNumber", 47i64)
        .result_raw()
        .await?;

    let test = api.dump_table("Test").await?;
    let first_row = test.get(0).unwrap();
    assert_eq!(
        format!("{:?} {:?}", first_row.get("id"), first_row.get("serialNumber")),
        r#"Some(Text("abcd")) Some(Integer(47))"#
    );

    let original_schema = api.assert_schema().await?.into_schema();

    let dm2 = r#"
        model Test {
            id String @id
            serialNumber String
        }
    "#;

    let expected_warning = MigrationWarning {
        description: "You are about to alter the column `serialNumber` on the `Test` table, which still contains 1 values. The data in that column may be lost.".to_owned(),
    };

    // Apply once without forcing
    {
        let result = api.infer_apply(dm2).send().await?.into_inner();

        assert_eq!(result.warnings, &[expected_warning.clone()]);

        api.assert_schema().await?.assert_equals(&original_schema)?;
    }

    // Force apply
    {
        let result = api.infer_apply(dm2).force(Some(true)).send().await?.into_inner();

        assert_eq!(result.warnings, &[expected_warning]);

        api.assert_schema().await?.assert_table("Test", |table| {
            table.assert_column("serialNumber", |col| col.assert_type_is_string())
        })?;

        let test = api.dump_table("Test").await?;
        let first_row = test.get(0).unwrap();
        assert_eq!(
            format!("{:?} {:?}", first_row.get("id"), first_row.get("serialNumber")),
            r#"Some(Text("abcd")) Some(Text("47"))"#
        );
    }

    Ok(())
}

#[test_each_connector(capabilities("enums"))]
async fn enum_variants_can_be_added_without_data_loss(api: &TestApi) -> TestResult {
    let dm1 = r#"
        model Cat {
            id String @id
            mood Mood
        }

        model Human {
            id String @id
            mood Mood
        }

        enum Mood {
            HAPPY
            HUNGRY
        }
    "#;

    api.infer_apply(dm1)
        .migration_id(Some("initial-setup"))
        .send()
        .await?
        .assert_green()?;

    {
        let cat_inserts = quaint::ast::Insert::multi_into(api.render_table_name("Cat"), vec!["id", "mood"])
            .values((
                Value::Text(Cow::Borrowed("felix")),
                Value::Enum(Cow::Borrowed("HUNGRY")),
            ))
            .values((
                Value::Text(Cow::Borrowed("mittens")),
                Value::Enum(Cow::Borrowed("HAPPY")),
            ));

        api.database().query(cat_inserts.into()).await?;
    }

    let dm2 = r#"
        model Cat {
            id String @id
            mood Mood
        }

        model Human {
            id String @id
            mood Mood
        }

        enum Mood {
            ABSOLUTELY_FABULOUS
            HAPPY
            HUNGRY
        }
    "#;

    api.infer_apply(dm2)
        .migration_id(Some("add-absolutely-fabulous-variant"))
        .send()
        .await?
        .assert_green()?;

    // Assertions
    {
        let cat_data = api.dump_table("Cat").await?;
        let cat_data: Vec<Vec<quaint::ast::Value>> =
            cat_data.into_iter().map(|row| row.into_iter().collect()).collect();

        let expected_cat_data = if api.sql_family().is_mysql() {
            vec![
                vec![Value::Text("felix".into()), Value::Text("HUNGRY".into())],
                vec![Value::Text("mittens".into()), Value::Text("HAPPY".into())],
            ]
        } else {
            vec![
                vec![Value::Text("felix".into()), Value::Enum("HUNGRY".into())],
                vec![Value::Text("mittens".into()), Value::Enum("HAPPY".into())],
            ]
        };

        assert_eq!(cat_data, expected_cat_data);

        let human_data = api.dump_table("Human").await?;
        let human_data: Vec<Vec<Value>> = human_data.into_iter().map(|row| row.into_iter().collect()).collect();
        let expected_human_data: Vec<Vec<Value>> = Vec::new();
        assert_eq!(human_data, expected_human_data);

        if api.sql_family().is_mysql() {
            api.assert_schema()
                .await?
                .assert_enum("Cat_mood", |enm| {
                    enm.assert_values(&["HAPPY", "HUNGRY", "ABSOLUTELY_FABULOUS"])
                })?
                .assert_enum("Human_mood", |enm| {
                    enm.assert_values(&["HAPPY", "HUNGRY", "ABSOLUTELY_FABULOUS"])
                })?;
        } else {
            api.assert_schema().await?.assert_enum("Mood", |enm| {
                enm.assert_values(&["ABSOLUTELY_FABULOUS", "HAPPY", "HUNGRY"])
            })?;
        };
    }

    Ok(())
}

#[test_each_connector(capabilities("enums"))]
async fn enum_variants_can_be_dropped_without_data_loss(api: &TestApi) -> TestResult {
    let dm1 = r#"
        model Cat {
            id String @id
            mood Mood
        }

        model Human {
            id String @id
            mood Mood
        }

        enum Mood {
            OUTRAGED
            HAPPY
            HUNGRY
        }
    "#;

    api.infer_apply(dm1)
        .migration_id(Some("initial-setup"))
        .send()
        .await?
        .assert_green()?;

    {
        let cat_inserts = quaint::ast::Insert::multi_into(api.render_table_name("Cat"), &["id", "mood"])
            .values((
                Value::Text(Cow::Borrowed("felix")),
                Value::Enum(Cow::Borrowed("HUNGRY")),
            ))
            .values((
                Value::Text(Cow::Borrowed("mittens")),
                Value::Enum(Cow::Borrowed("HAPPY")),
            ));

        api.database().query(cat_inserts.into()).await?;
    }

    let dm2 = r#"
        model Cat {
            id String @id
            mood Mood
        }

        model Human {
            id String @id
            mood Mood
        }

        enum Mood {
            HAPPY
            HUNGRY
        }
    "#;

    api.infer_apply(dm2)
        .migration_id(Some("add-absolutely-fabulous-variant"))
        .send()
        .await?
        .assert_green()?;

    // Assertions
    {
        let cat_data = api.dump_table("Cat").await?;
        let cat_data: Vec<Vec<quaint::ast::Value>> =
            cat_data.into_iter().map(|row| row.into_iter().collect()).collect();

        let expected_cat_data = if api.sql_family().is_mysql() {
            vec![
                vec![Value::Text("felix".into()), Value::Text("HUNGRY".into())],
                vec![Value::Text("mittens".into()), Value::Text("HAPPY".into())],
            ]
        } else {
            vec![
                vec![Value::Text("felix".into()), Value::Enum("HUNGRY".into())],
                vec![Value::Text("mittens".into()), Value::Enum("HAPPY".into())],
            ]
        };

        assert_eq!(cat_data, expected_cat_data);

        let human_data = api.dump_table("Human").await?;
        let human_data: Vec<Vec<Value>> = human_data.into_iter().map(|row| row.into_iter().collect()).collect();
        let expected_human_data: Vec<Vec<Value>> = Vec::new();
        assert_eq!(human_data, expected_human_data);

        if api.sql_family().is_mysql() {
            api.assert_schema()
                .await?
                .assert_enum("Cat_mood", |enm| enm.assert_values(&["HAPPY", "HUNGRY"]))?
                .assert_enum("Human_mood", |enm| enm.assert_values(&["HAPPY", "HUNGRY"]))?;
        } else {
            api.assert_schema()
                .await?
                .assert_enum("Mood", |enm| enm.assert_values(&["HAPPY", "HUNGRY"]))?;
        };
    }

    Ok(())
}
