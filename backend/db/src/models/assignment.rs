//! Entity and business logic for managing assignments.
//!
//! This module defines the `Assignment` model, its relations, and
//! methods for creating, editing, and filtering assignments.

use sea_orm::entity::prelude::*;
use sea_orm::{EntityTrait, JsonValue};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use strum::{Display, EnumIter, EnumString};

/// Assignment model representing the `assignments` table in the database.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "assignments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub module_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub assignment_type: AssignmentType,
    pub status: Status,
    pub available_from: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    #[sea_orm(column_type = "Json", nullable)]
    pub config: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Defines the relationship between `Assignment` and `Module`.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::module::Entity",
        from = "Column::ModuleId",
        to = "super::module::Column::Id"
    )]
    Module,
}

#[derive(Debug, Clone, PartialEq, Display, EnumIter, EnumString, Serialize, Deserialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "assignment_type_enum")]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum AssignmentType {
    #[sea_orm(string_value = "assignment")]
    Assignment,

    #[sea_orm(string_value = "practical")]
    Practical,
}

#[derive(Debug, Clone, PartialEq, Display, EnumIter, EnumString, Serialize, Deserialize, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "assignment_status_enum")]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum Status {
    #[sea_orm(string_value = "setup")]
    Setup,
    #[sea_orm(string_value = "ready")]
    Ready,
    #[sea_orm(string_value = "open")]
    Open,
    #[sea_orm(string_value = "closed")]
    Closed,
    #[sea_orm(string_value = "archived")]
    Archived,
}

/// Detailed report of assignment readiness state.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReadinessReport {
    pub config_present: bool,
    pub tasks_present: bool,
    pub main_present: bool,
    pub memo_present: bool,
    pub makefile_present: bool,
    pub memo_output_present: bool,
    pub mark_allocator_present: bool,
}

impl ReadinessReport {
    /// Convenience helper: true if all components are present.
    pub fn is_ready(&self) -> bool {
        self.config_present
            && self.tasks_present
            && self.main_present
            && self.memo_present
            && self.makefile_present
            && self.memo_output_present
            && self.mark_allocator_present
    }
}

impl ActiveModelBehavior for ActiveModel {}
impl Model {}