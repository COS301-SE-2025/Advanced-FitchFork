use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m202509120001_create_moss_reports.rs"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("moss_reports"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("assignment_id"))
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("report_url")).text().not_null())
                    .col(
                        ColumnDef::new(Alias::new("generated_at"))
                            .timestamp()
                            .not_null()
                            .default(Expr::cust("CURRENT_TIMESTAMP")),
                    )
                    // NEW: optional human description of this run
                    .col(ColumnDef::new(Alias::new("description")).text().null())
                    // archive details
                    .col(
                        ColumnDef::new(Alias::new("has_archive"))
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Alias::new("archive_generated_at"))
                            .timestamp()
                            .null(),
                    )
                    // generation filter metadata
                    .col(
                        ColumnDef::new(Alias::new("filter_mode"))
                            .string()
                            .not_null()
                            .default("all"),
                    )
                    .col(
                        ColumnDef::new(Alias::new("filter_patterns"))
                            .json_binary()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_moss_reports_assignment")
                            .from(Alias::new("moss_reports"), Alias::new("assignment_id"))
                            .to(Alias::new("assignments"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("moss_reports")).to_owned())
            .await
    }
}
