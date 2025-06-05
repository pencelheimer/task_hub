use sea_orm_migration::prelude::{extension::postgres::Type, *};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Create the 'access_level_enum' enum
        m.create_type(
            Type::create()
                .as_enum(Alias::new("access_level_enum"))
                .values(vec![
                    Alias::new("View"),
                    Alias::new("AddSolution"),
                    Alias::new("Edit"),
                    Alias::new("AddUser"),
                    Alias::new("FullAccess"),
                ])
                .to_owned(),
        )
        .await?;

        // Create the 'accesses' table
        m.create_table(
            Table::create()
                .table(Alias::new("accesses"))
                .col(
                    ColumnDef::new(Alias::new("id"))
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(Alias::new("accesslevel"))
                        .enumeration(
                            Alias::new("access_level_enum"),
                            vec![
                                Alias::new("View"),
                                Alias::new("AddSolution"),
                                Alias::new("Edit"),
                                Alias::new("AddUser"),
                                Alias::new("FullAccess"),
                            ],
                        )
                        .not_null(),
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
                // Foreign Key for 'user'
                .col(ColumnDef::new(Alias::new("user_id")).integer().not_null())
                .foreign_key(
                    ForeignKey::create()
                        .from_tbl(Alias::new("accesses"))
                        .from_col(Alias::new("user_id"))
                        .to_tbl(Alias::new("users"))
                        .to_col(Alias::new("id"))
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                // Foreign Key for 'task'
                .col(ColumnDef::new(Alias::new("task_id")).integer().not_null())
                .foreign_key(
                    ForeignKey::create()
                        .from_tbl(Alias::new("accesses"))
                        .from_col(Alias::new("task_id"))
                        .to_tbl(Alias::new("tasks"))
                        .to_col(Alias::new("id"))
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(Alias::new("accesses")).to_owned())
            .await?;

        m.drop_type(
            Type::drop()
                .name(Alias::new("access_level_enum"))
                .to_owned(),
        )
        .await
    }
}
