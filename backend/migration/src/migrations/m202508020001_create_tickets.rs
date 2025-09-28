use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m202508020001_create_tickets"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("tickets"))
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
                    .col(
                        ColumnDef::new(Alias::new("user_id"))
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("title")).text().not_null())
                    .col(ColumnDef::new(Alias::new("description")).text().not_null())
                    .col(
                        ColumnDef::new(Alias::new("status"))
                            .enumeration(
                                Alias::new("ticket_status"),
                                vec![Alias::new("open"), Alias::new("closed")],
                            )
                            .not_null()
                            .default("open"),
                    )
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
                            .from(Alias::new("tickets"), Alias::new("assignment_id"))
                            .to(Alias::new("assignments"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("tickets"), Alias::new("user_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("tickets")).to_owned())
            .await
    }
}
