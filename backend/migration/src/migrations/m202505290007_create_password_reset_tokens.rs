use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m202505290006_create_password_reset_token"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("password_reset_tokens"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("user_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("token")).string().not_null())
                    .col(ColumnDef::new(Alias::new("expires_at")).timestamp().not_null())
                    .col(ColumnDef::new(Alias::new("used")).boolean().not_null().default(false))
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp().not_null().default(Expr::cust("CURRENT_TIMESTAMP")))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_password_reset_token_user")
                            .from(Alias::new("password_reset_tokens"), Alias::new("user_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("password_reset_tokens")).to_owned())
            .await
    }
}