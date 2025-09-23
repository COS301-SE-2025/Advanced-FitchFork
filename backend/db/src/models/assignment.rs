//! Entity and business logic for managing assignments.
//!
//! This module defines the `Assignment` model, its relations, and
//! methods for creating, editing, and filtering assignments.

use crate::models::moss_report;
use chrono::{DateTime, Utc};
use sea_orm::EntityTrait;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};
use util::execution_config::SubmissionMode;

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

    #[sea_orm(has_many = "super::moss_report::Entity")]
    MossReports,
}

impl Related<moss_report::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MossReports.def()
    }
}

#[derive(
    Debug, Clone, PartialEq, Display, EnumIter, EnumString, Serialize, Deserialize, DeriveActiveEnum,
)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "assignment_type_enum"
)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum AssignmentType {
    #[sea_orm(string_value = "assignment")]
    Assignment,

    #[sea_orm(string_value = "practical")]
    Practical,
}

#[derive(
    Debug, Clone, PartialEq, Display, EnumIter, EnumString, Serialize, Deserialize, DeriveActiveEnum,
)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "assignment_status_enum"
)]
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
    pub submission_mode: SubmissionMode,
    pub config_present: bool,
    pub tasks_present: bool,
    pub main_present: bool,
    pub interpreter_present: bool,
    pub memo_present: bool,
    pub makefile_present: bool,
    pub memo_output_present: bool,
    pub mark_allocator_present: bool,
}

impl ReadinessReport {
    /// Readiness is conditional:
    /// - Manual  -> require main
    /// - GATLAM  -> require interpreter
    /// - Others  -> neither main nor interpreter are required
    pub fn is_ready(&self) -> bool {
        let common_ok = self.config_present
            && self.tasks_present
            && self.memo_present
            && self.makefile_present
            && self.memo_output_present
            && self.mark_allocator_present;

        if !common_ok {
            return false;
        }

        match self.submission_mode {
            SubmissionMode::Manual => self.main_present,
            SubmissionMode::GATLAM => self.interpreter_present,
            _ => true,
        }
    }
}

/// Quick summary object for attempts.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AttemptsSummary {
    pub used: u32,
    pub max: u32,
    pub remaining: u32,
    pub limit_attempts: bool,
}

impl ActiveModelBehavior for ActiveModel {}
