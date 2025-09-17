use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m202505290003_create_user_module_roles"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("user_module_roles"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("user_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("module_id")).integer().not_null())
                    .col(
                        ColumnDef::new(Alias::new("role"))
                            .enumeration(
                                Alias::new("user_module_role_type"),
                                vec![
                                    Alias::new("lecturer"),
                                    Alias::new("assistant_lecturer"),
                                    Alias::new("tutor"),
                                    Alias::new("student"),
                                ],
                            )
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(Alias::new("user_id"))
                            .col(Alias::new("module_id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("user_module_roles"), Alias::new("user_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("user_module_roles"), Alias::new("module_id"))
                            .to(Alias::new("modules"), Alias::new("id"))
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
                    .table(Alias::new("user_module_roles"))
                    .to_owned(),
            )
            .await
    }
}
