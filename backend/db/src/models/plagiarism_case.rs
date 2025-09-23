//! Entity and business logic for managing plagiarism cases.

use chrono::{DateTime, Utc};
use sea_orm::EntityTrait;
use sea_orm::entity::prelude::*;

/// Represents a detected plagiarism case between two submissions.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "plagiarism_cases")]
pub struct Model {
    /// Primary key for the plagiarism case.
    #[sea_orm(primary_key)]
    pub id: i64,

    /// ID of the assignment this case belongs to.
    pub assignment_id: i64,

    /// ID of the first submission involved in the case.
    pub submission_id_1: i64,

    /// ID of the second submission involved in the case.
    pub submission_id_2: i64,

    /// Optional: the MOSS report this case came from.
    /// NOTE: Nullable so that deleting a report does NOT delete cases.
    pub report_id: Option<i64>,

    /// Description of the plagiarism incident.
    pub description: String,

    /// The review status of the case.
    pub status: Status,

    /// Similarity percentage (0–100) as float.
    pub similarity: f32,

    /// Total lines matched across the pair (parsed from MOSS).
    pub lines_matched: i64,

    /// Timestamp when the case was created.
    pub created_at: DateTime<Utc>,

    /// Timestamp when the case was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Defines the possible review statuses for a plagiarism case.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    sea_orm::strum::Display,
    sea_orm::strum::EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "Text")]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum Status {
    /// The case has not yet been reviewed.
    #[sea_orm(string_value = "review")]
    Review,
    /// The case has been flagged for potential plagiarism.
    #[sea_orm(string_value = "flagged")]
    Flagged,
    /// The case has been reviewed and cleared.
    #[sea_orm(string_value = "reviewed")]
    Reviewed,
}

/// Defines relationships to assignment submissions and the report.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::assignment_submission::Entity",
        from = "Column::SubmissionId1",
        to = "super::assignment_submission::Column::Id"
    )]
    Submission1,

    #[sea_orm(
        belongs_to = "super::assignment_submission::Entity",
        from = "Column::SubmissionId2",
        to = "super::assignment_submission::Column::Id"
    )]
    Submission2,

    #[sea_orm(
        belongs_to = "super::moss_report::Entity",
        from = "Column::ReportId",
        to = "super::moss_report::Column::Id"
    )]
    Report,
}

impl ActiveModelBehavior for ActiveModel {}
