use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m202505290004_create_assignments"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("assignments"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Alias::new("module_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("name")).string().not_null())
                    .col(ColumnDef::new(Alias::new("description")).string().null())
                    .col(
                        ColumnDef::new(Alias::new("assignment_type"))
                            .enumeration(
                                Alias::new("assignment_type_enum"),
                                vec![
                                    Alias::new("assignment"),
                                    Alias::new("practical"),
                                ],
                            )
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("available_from")).timestamp().not_null())
                    .col(ColumnDef::new(Alias::new("due_date")).timestamp().not_null())
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp().not_null().default(Expr::cust("CURRENT_TIMESTAMP")))
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp().not_null().default(Expr::cust("CURRENT_TIMESTAMP")))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("assignments"), Alias::new("module_id"))
                            .to(Alias::new("modules"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("assignments")).to_owned())
            .await
    }
}
