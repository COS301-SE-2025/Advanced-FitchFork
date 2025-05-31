use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m202505290006_create_assignment_submission"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("assignment_submissions"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("assignment_id"))
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("user_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("filename")).string().not_null())
                    .col(ColumnDef::new(Alias::new("path")).string().not_null())
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp()
                            .not_null()
                            .default(Expr::cust("CURRENT_TIMESTAMP")),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
                            .timestamp()
                            .not_null()
                            .default(Expr::cust("CURRENT_TIMESTAMP")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                Alias::new("assignment_submissions"),
                                Alias::new("assignment_id"),
                            )
                            .to(Alias::new("assignments"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("assignment_submissions"), Alias::new("user_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("assignment_submissions"))
                    .to_owned(),
            )
            .await
    }
}
