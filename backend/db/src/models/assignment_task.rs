use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::EntityTrait;
use strum::EnumIter;

/// Assignment task model representing the `assignment_tasks` table.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "assignment_tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub assignment_id: i64,
    pub task_number: i64,
    pub name: String,
    pub command: String,
    pub code_coverage: bool,
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
}

impl ActiveModelBehavior for ActiveModel {}
impl Model {}