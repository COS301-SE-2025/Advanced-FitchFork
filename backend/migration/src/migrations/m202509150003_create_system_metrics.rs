// migrations/m202509150003_create_system_metrics.rs
use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum SystemMetrics {
    Table,
    Id,
    CreatedAt,
    CpuAvg,
    MemPct,
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m202509150003_create_system_metrics"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SystemMetrics::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SystemMetrics::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SystemMetrics::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SystemMetrics::CpuAvg).float().not_null())
                    .col(
                        ColumnDef::new(SystemMetrics::MemPct)
                            .float()
                            .not_null()
                            .default(0.0),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SystemMetrics::Table).to_owned())
            .await
    }
}
