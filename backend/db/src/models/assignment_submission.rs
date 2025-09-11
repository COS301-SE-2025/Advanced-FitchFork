use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::EntityTrait;
use crate::models::user;

/// Represents a user's submission for a specific assignment.
///
/// Each submission is linked to one assignment and one user.
/// Timestamps are included to track when the submission was created and last updated.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "assignment_submissions")]
pub struct Model {
    /// Primary key of the submission.
    #[sea_orm(primary_key)]
    pub id: i64,
    /// ID of the related assignment.
    pub assignment_id: i64,
    /// ID of the user who submitted the assignment.
    pub user_id: i64,
    /// Attempt number
    pub attempt: i64,
    /// The score earned by the user.
    pub earned: i64,
    /// The total possible score.
    pub total: i64,
    /// Is this submission a practice submission?
    pub is_practice: bool,
    /// Whether this submission should be ignored for grading/analytics.
    pub ignored: bool,
    /// The original filename uploaded by the user.
    pub filename: String,
    /// The hash of the submitted files.
    pub file_hash: String,
    /// Relative file path from the storage root.
    pub path: String,
    /// Timestamp when the submission was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the submission was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Defines relationships between `assignment_submissions` and other tables.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Link to the related assignment.
    #[sea_orm(
        belongs_to = "super::assignment::Entity",
        from = "Column::AssignmentId",
        to = "super::assignment::Column::Id"
    )]
    Assignment,

    /// Link to the user who submitted the assignment.
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}