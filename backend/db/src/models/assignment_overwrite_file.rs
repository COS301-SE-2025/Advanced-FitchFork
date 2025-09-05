use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::EntityTrait;

/// Represents a file used to overwrite specific parts of an assignment during evaluation.
/// Includes metadata such as its related assignment, task, filename, and storage path.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "assignment_overwrite_files")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub assignment_id: i64,
    pub task_id: i64,
    pub filename: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::assignment::Entity",
        from = "Column::AssignmentId",
        to = "super::assignment::Column::Id"
    )]
    Assignment,

    #[sea_orm(
        belongs_to = "super::assignment_task::Entity",
        from = "Column::TaskId",
        to = "super::assignment_task::Column::TaskNumber"
    )]
    AssignmentTask,
}

impl ActiveModelBehavior for ActiveModel {}
impl Model {}