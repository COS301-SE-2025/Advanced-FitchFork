use crate::models::user;
use chrono::{DateTime, Utc};
use sea_orm::EntityTrait;
use sea_orm::entity::prelude::*;

/// Represents the status of a submission throughout its lifecycle
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "submission_status_enum"
)]
pub enum SubmissionStatus {
    /// Waiting for execution/marking
    #[sea_orm(string_value = "queued")]
    Queued,
    /// Code is compiling/building
    #[sea_orm(string_value = "running")]
    Running,
    /// Marking in progress
    #[sea_orm(string_value = "grading")]
    Grading,
    /// Marking complete (success)
    #[sea_orm(string_value = "graded")]
    Graded,
    /// File save error
    #[sea_orm(string_value = "failed_upload")]
    FailedUpload,
    /// Compilation failure
    #[sea_orm(string_value = "failed_compile")]
    FailedCompile,
    /// Runtime/test execution failure
    #[sea_orm(string_value = "failed_execution")]
    FailedExecution,
    /// Marking logic failure
    #[sea_orm(string_value = "failed_grading")]
    FailedGrading,
    /// Unexpected internal error
    #[sea_orm(string_value = "failed_internal")]
    FailedInternal,
    /// Disallowed code detected (policy violation)
    #[sea_orm(string_value = "failed_disallowed_code")]
    FailedDisallowedCode,
}

impl Default for SubmissionStatus {
    fn default() -> Self {
        Self::Queued
    }
}

impl std::fmt::Display for SubmissionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_str = match self {
            SubmissionStatus::Queued => "queued",
            SubmissionStatus::Running => "running",
            SubmissionStatus::Grading => "grading",
            SubmissionStatus::Graded => "graded",
            SubmissionStatus::FailedUpload => "failed_upload",
            SubmissionStatus::FailedCompile => "failed_compile",
            SubmissionStatus::FailedExecution => "failed_execution",
            SubmissionStatus::FailedGrading => "failed_grading",
            SubmissionStatus::FailedInternal => "failed_internal",
            SubmissionStatus::FailedDisallowedCode => "failed_disallowed_code",
        };
        write!(f, "{}", status_str)
    }
}

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
    /// The original filename uploaded by the user.
    pub filename: String,
    /// The hash of the submitted files.
    pub file_hash: String,
    /// Relative file path from the storage root.
    pub path: String,
    /// Is this submission a practice submission?
    pub is_practice: bool,
    /// Whether this submission should be ignored for grading/analytics.
    pub ignored: bool,
    /// Current status of the submission in the lifecycle.
    pub status: SubmissionStatus,
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
