use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m202505290012_create_plagiarism_cases"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("plagiarism_cases"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id"))
                        .big_integer()
                        .not_null()
                        .auto_increment()
                        .primary_key())
                    .col(ColumnDef::new(Alias::new("submission_id_1"))
                        .big_integer()
                        .not_null())
                    .col(ColumnDef::new(Alias::new("submission_id_2"))
                        .big_integer()
                        .not_null())
                    .col(ColumnDef::new(Alias::new("description"))
                        .text()
                        .not_null())
                    .col(ColumnDef::new(Alias::new("status"))
                        .string()
                        .not_null()
                        .default("review"))
                    .col(ColumnDef::new(Alias::new("created_at"))
                        .timestamp()
                        .not_null()
                        .default(Expr::cust("CURRENT_TIMESTAMP")))
                    .col(ColumnDef::new(Alias::new("updated_at"))
                        .timestamp()
                        .not_null()
                        .default(Expr::cust("CURRENT_TIMESTAMP")))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_plagiarism_submission1")
                            .from(Alias::new("plagiarism_cases"), Alias::new("submission_id_1"))
                            .to(Alias::new("assignment_submissions"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_plagiarism_submission2")
                            .from(Alias::new("plagiarism_cases"), Alias::new("submission_id_2"))
                            .to(Alias::new("assignment_submissions"), Alias::new("id"))
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
                    .table(Alias::new("plagiarism_cases"))
                    .to_owned(),
            )
            .await
    }
}
