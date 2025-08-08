use sea_orm_migration::prelude::*;
use sea_query::{Expr};


#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum Grades {
    Table,
    Id,
    AssignmentId,
    StudentId,
    SubmissionId,
    Score,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
 enum Assignments {
    Table,
    Id,
}


#[derive(Iden)]
enum Users {
    Table,
    Id,
}


#[derive(Iden)]
enum Submissions {
    Table,
    Id,
}


#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Grades::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Grades::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Grades::AssignmentId).big_integer().not_null())
                    .col(ColumnDef::new(Grades::StudentId).big_integer().not_null())
                    .col(ColumnDef::new(Grades::SubmissionId).big_integer().null())
                    .col(
                        ColumnDef::new(Grades::Score)
                            .float()
                            .not_null()
                            .default(0.0_f64),
                    )
                    .col(
                        ColumnDef::new(Grades::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::cust("CURRENT_TIMESTAMP")),
                    )
                    .col(
                        ColumnDef::new(Grades::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::cust("CURRENT_TIMESTAMP")),
                    )
                    // FKs
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_grades_assignment")
                            .from(Grades::Table, Grades::AssignmentId)
                            .to(Assignments::Table, Assignments::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_grades_student")
                            .from(Grades::Table, Grades::StudentId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_grades_submission")
                            .from(Grades::Table, Grades::SubmissionId)
                            .to(Submissions::Table, Submissions::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Grades::Table).to_owned())
            .await
    }
}