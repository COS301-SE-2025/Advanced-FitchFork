use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m202505290001_create_users"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("users"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Alias::new("username")).string().not_null().unique_key())
                    .col(ColumnDef::new(Alias::new("email")).string().not_null().unique_key())
                    .col(ColumnDef::new(Alias::new("password_hash")).string().not_null())
                    .col(ColumnDef::new(Alias::new("admin")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp().not_null().default(Expr::cust("CURRENT_TIMESTAMP")))
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp().not_null().default(Expr::cust("CURRENT_TIMESTAMP")))
                    .col(ColumnDef::new(Alias::new("profile_picture_path")).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("users")).to_owned())
            .await
    }
}
