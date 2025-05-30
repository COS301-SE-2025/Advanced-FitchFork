use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m202505290002_create_modules"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("modules"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Alias::new("code")).string().not_null())
                    .col(ColumnDef::new(Alias::new("year")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("description")).string().null())
                    .col(ColumnDef::new(Alias::new("credits")).integer().not_null().default(0))
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp().not_null().default(Expr::cust("CURRENT_TIMESTAMP")))
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp().not_null().default(Expr::cust("CURRENT_TIMESTAMP")))
                    .index(Index::create().col(Alias::new("code")).col(Alias::new("year")).unique())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("modules")).to_owned())
            .await
    }
}
