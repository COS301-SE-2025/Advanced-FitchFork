use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

/// Represents an interpreter file associated with an assignment,
/// including the command used to run it.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "interpreters")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    /// Foreign key reference to an assignment.
    pub assignment_id: i64,

    /// Original file name.
    pub filename: String,

    /// Relative path to the stored file from the storage root.
    pub path: String,

    /// Command to run this interpreter.
    pub command: String,

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
