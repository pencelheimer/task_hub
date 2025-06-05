use sea_orm_migration::prelude::{extension::postgres::Type, *};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_type(
            Type::create()
                .as_enum(Alias::new("task_visibility_enum"))
                .values(vec![
                    Alias::new("Private"),
                    Alias::new("Public"),
                    Alias::new("Paid"),
                ])
                .to_owned(),
        )
        .await?;

        m.create_table(
            Table::create()
                .table(Alias::new("tasks"))
                .col(
                    ColumnDef::new(Alias::new("id"))
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(Alias::new("name")).string().not_null())
                .col(
                    ColumnDef::new(Alias::new("visibility"))
                        .enumeration(
                            Alias::new("task_visibility_enum"),
                            vec![
                                Alias::new("Private"),
                                Alias::new("Public"),
                                Alias::new("Paid"),
                            ],
                        )
                        .not_null()
                        .default(Value::String(Some(Box::new("Private".to_owned())))),
                )
                .col(
                    ColumnDef::new(Alias::new("created_at"))
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Alias::new("updated_at"))
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .to_owned(),
        )
        .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("tasks")).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(Alias::new("task_visibility_enum"))
                    .to_owned(),
            )
            .await
    }
}
