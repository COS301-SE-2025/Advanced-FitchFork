// migration: create_attendance (updated)
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m202509080001_create_attendance"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // attendance_sessions
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("attendance_sessions"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("module_id"))
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("created_by"))
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("title")).string().not_null())
                    .col(
                        ColumnDef::new(Alias::new("active"))
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Alias::new("rotation_seconds"))
                            .integer()
                            .not_null()
                            .default(30),
                    )
                    // REMOVED: code_length
                    .col(
                        ColumnDef::new(Alias::new("secret"))
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("restrict_by_ip"))
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Alias::new("allowed_ip_cidr"))
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("created_from_ip"))
                            .string()
                            .null(),
                    )
                    // REMOVED: allow_manual_entry
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
                            .name("fk_att_sess_module")
                            .from(Alias::new("attendance_sessions"), Alias::new("module_id"))
                            .to(Alias::new("modules"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_att_sess_user")
                            .from(Alias::new("attendance_sessions"), Alias::new("created_by"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // attendance_records
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("attendance_records"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("session_id"))
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("user_id"))
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("taken_at"))
                            .timestamp()
                            .not_null()
                            .default(Expr::cust("CURRENT_TIMESTAMP")),
                    )
                    .col(ColumnDef::new(Alias::new("ip_address")).string().null())
                    // REMOVED: method
                    .col(
                        ColumnDef::new(Alias::new("token_window"))
                            .big_integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(Alias::new("session_id"))
                            .col(Alias::new("user_id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_att_rec_session")
                            .from(Alias::new("attendance_records"), Alias::new("session_id"))
                            .to(Alias::new("attendance_sessions"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_att_rec_user")
                            .from(Alias::new("attendance_records"), Alias::new("user_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("attendance_records"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("attendance_sessions"))
                    .to_owned(),
            )
            .await
    }
}
