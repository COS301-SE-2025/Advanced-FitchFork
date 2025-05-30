use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

/// Represents a submission for an assignment by a user.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "assignment_submissions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    /// Foreign key to the assignment.
    pub assignment_id: i64,

    /// Foreign key to the user who submitted.
    pub user_id: i64,

    /// Grade awarded to this submission.
    pub grade: i32,

    /// Feedback comments for the submission.
    pub feedback: String,

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
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl ActiveModelBehavior for ActiveModel {}
