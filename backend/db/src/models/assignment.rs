//! Entity and business logic for managing assignments.
//!
//! This module defines the `Assignment` model, its relations, and
//! methods for creating, editing, and filtering assignments.

use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DbErr, EntityTrait, IntoActiveModel,
    JsonValue, PaginatorTrait, QueryFilter, QueryOrder, Set, QuerySelect
};
use crate::models::assignment_file::{Model as AssignmentFileModel, FileType};
use crate::models::assignment_task::{Entity as TaskEntity, Column as TaskColumn};
use chrono::{DateTime, Utc};
use std::{env, fs, path::PathBuf};
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

impl ActiveModelBehavior for ActiveModel {}

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

impl Model {
    pub async fn create(
        db: &DatabaseConnection,
        module_id: i64,
        name: &str,
        description: Option<&str>,
        assignment_type: AssignmentType,
        available_from: DateTime<Utc>,
        due_date: DateTime<Utc>,
    ) -> Result<Self, DbErr> {
        let active = ActiveModel {
            module_id: Set(module_id),
            name: Set(name.to_string()),
            description: Set(description.map(|d| d.to_string())),
            assignment_type: Set(assignment_type),
            status: Set(Status::Setup), // always starts as Setup
            available_from: Set(available_from),
            due_date: Set(due_date),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        active.insert(db).await
    }

    pub async fn edit(
        db: &DatabaseConnection,
        id: i64,
        module_id: i64,
        name: &str,
        description: Option<&str>,
        assignment_type: AssignmentType,
        available_from: DateTime<Utc>,
        due_date: DateTime<Utc>,
    ) -> Result<Self, DbErr> {
        let mut assignment = Entity::find()
            .filter(Column::Id.eq(id))
            .filter(Column::ModuleId.eq(module_id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Assignment not found".to_string()))?
            .into_active_model();

        assignment.name = Set(name.to_string());
        assignment.description = Set(description.map(|d| d.to_string()));
        assignment.assignment_type = Set(assignment_type);
        assignment.available_from = Set(available_from);
        assignment.due_date = Set(due_date);
        assignment.updated_at = Set(Utc::now());

        assignment.update(db).await
    }

    pub async fn delete(db: &DatabaseConnection, id: i32, module_id: i32) -> Result<(), DbErr> {
        let Some(model) = Entity::find()
            .filter(Column::Id.eq(id))
            .filter(Column::ModuleId.eq(module_id))
            .one(db)
            .await?
        else {
            return Err(DbErr::RecordNotFound(format!(
                "Assignment {id} in module {module_id} not found"
            )));
        };

        let active = model.into_active_model();
        active.delete(db).await?;

        let storage_root = env::var("ASSIGNMENT_STORAGE_ROOT")
            .unwrap_or_else(|_| "data/assignment_files".to_string());

        let assignment_dir = PathBuf::from(storage_root)
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{id}"));

        if assignment_dir.exists() {
            if let Err(e) = fs::remove_dir_all(&assignment_dir) {
                eprintln!("Warning: Failed to delete assignment directory {:?}: {}", assignment_dir, e);
            }
        }

        Ok(())
    }

    pub async fn filter(
        db: &DatabaseConnection,
        page: u64,
        per_page: u64,
        sort_by: Option<String>,
        query: Option<String>,
    ) -> Result<Vec<Self>, DbErr> {
        let mut query_builder = Entity::find();

        if let Some(q) = query {
            let pattern = format!("%{}%", q.to_lowercase());
            query_builder = query_builder.filter(
                Condition::any()
                    .add(Expr::cust("LOWER(name)").like(&pattern))
                    .add(Expr::cust("LOWER(description)").like(&pattern)),
            );
        }

        if let Some(sort) = sort_by {
            let (column, asc) = if sort.starts_with('-') {
                (&sort[1..], false)
            } else {
                (sort.as_str(), true)
            };

            query_builder = match column {
                "name" => {
                    if asc {
                        query_builder.order_by_asc(Column::Name)
                    } else {
                        query_builder.order_by_desc(Column::Name)
                    }
                }
                "due_date" => {
                    if asc {
                        query_builder.order_by_asc(Column::DueDate)
                    } else {
                        query_builder.order_by_desc(Column::DueDate)
                    }
                }
                "available_from" => {
                    if asc {
                        query_builder.order_by_asc(Column::AvailableFrom)
                    } else {
                        query_builder.order_by_desc(Column::AvailableFrom)
                    }
                }
                _ => query_builder,
            };
        }

        query_builder.paginate(db, per_page).fetch_page(page - 1).await
    }

    /// Computes a detailed readiness report for an assignment by checking if all required components are present.
    ///
    /// This function verifies the presence of all expected resources for an assignment:
    /// - At least one task is defined in the database.
    /// - A configuration file exists.
    /// - A main file exists.
    /// - A memo file exists.
    /// - A makefile exists.
    /// - A memo output file exists.
    /// - A JSON mark allocator file exists.
    ///
    /// The returned [`ReadinessReport`] (or [`AssignmentReadiness`]) contains boolean flags for each component
    /// and an `is_ready` field indicating whether the assignment is fully ready
    /// (i.e., suitable to transition from `Setup` to `Ready` state).
    ///
    /// This function only checks readiness — it does **not** modify the assignment's status in the database.
    ///
    /// # Arguments
    /// * `db` - A reference to the database connection.
    /// * `module_id` - The ID of the module to which the assignment belongs.
    /// * `assignment_id` - The ID of the assignment to check.
    ///
    /// # Returns
    /// * `Ok(ReadinessReport)` with per-component readiness details.
    /// * `Err(DbErr)` if a database error occurs while checking tasks.
    ///
    /// # Notes
    /// File presence is checked on the file system under the expected directories for each file type.
    /// Missing directories or I/O errors are treated as absence of the respective component.
    pub async fn compute_readiness_report(
        db: &DatabaseConnection,
        module_id: i64,
        assignment_id: i64,
    ) -> Result<ReadinessReport, DbErr> {
        let config_present = AssignmentFileModel::full_directory_path(
            module_id,
            assignment_id,
            &FileType::Config,
        )
        .read_dir()
        .map(|mut it| it.any(|f| f.is_ok()))
        .unwrap_or(false);

        let tasks_present = TaskEntity::find()
            .filter(TaskColumn::AssignmentId.eq(assignment_id))
            .limit(1)
            .all(db)
            .await
            .map(|tasks| !tasks.is_empty())
            .unwrap_or(false);

        let main_present = AssignmentFileModel::full_directory_path(
            module_id,
            assignment_id,
            &FileType::Main,
        )
        .read_dir()
        .map(|mut it| it.any(|f| f.is_ok()))
        .unwrap_or(false);

        let memo_present = AssignmentFileModel::full_directory_path(
            module_id,
            assignment_id,
            &FileType::Memo,
        )
        .read_dir()
        .map(|mut it| it.any(|f| f.is_ok()))
        .unwrap_or(false);

        let makefile_present = AssignmentFileModel::full_directory_path(
            module_id,
            assignment_id,
            &FileType::Makefile,
        )
        .read_dir()
        .map(|mut it| it.any(|f| f.is_ok()))
        .unwrap_or(false);

        let memo_output_present = {
            let base_path = AssignmentFileModel::storage_root()
                .join(format!("module_{}", module_id))
                .join(format!("assignment_{}", assignment_id))
                .join("memo_output");

            if let Ok(entries) = fs::read_dir(&base_path) {
                entries.flatten().any(|entry| entry.path().is_file())
            } else {
                false
            }
        };

        let mark_allocator_present = AssignmentFileModel::full_directory_path(
            module_id,
            assignment_id,
            &FileType::MarkAllocator,
        )
        .read_dir()
        .map(|it| {
            it.flatten()
                .any(|f| f.path().extension().map(|e| e == "json").unwrap_or(false))
        })
        .unwrap_or(false);

        Ok(ReadinessReport {
            config_present,
            tasks_present,
            main_present,
            memo_present,
            makefile_present,
            memo_output_present,
            mark_allocator_present,
        })
    }

    /// Attempts to transition an assignment to `Ready` state if all readiness conditions are met.
    ///
    /// This function:
    /// - Computes a full readiness report for the assignment.
    /// - If all components are present (`is_ready` == true), it checks the current status.
    /// - If the current status is `Setup`, it updates the status to `Ready` and updates `updated_at`.
    /// - If already in `Ready`, `Open`, etc., it does not change the status.
    ///
    /// # Arguments
    /// * `db` - A reference to the database connection.
    /// * `module_id` - The ID of the module to which the assignment belongs.
    /// * `assignment_id` - The ID of the assignment.
    ///
    /// # Returns
    /// * `Ok(true)` if the assignment is fully ready (regardless of whether the status was updated).
    /// * `Ok(false)` if the assignment is not ready.
    /// * `Err(DbErr)` if a database error occurs.
    pub async fn try_transition_to_ready(
        db: &DatabaseConnection,
        module_id: i64,
        assignment_id: i64,
    ) -> Result<bool, DbErr> {
        let report = Self::compute_readiness_report(db, module_id, assignment_id).await?;

        if report.is_ready() {
            let mut active = Entity::find()
                .filter(Column::Id.eq(assignment_id))
                .filter(Column::ModuleId.eq(module_id))
                .one(db)
                .await?
                .ok_or(DbErr::RecordNotFound("Assignment not found".into()))?
                .into_active_model();

            if active.status.as_ref() == &Status::Setup {
                active.status = Set(Status::Ready);
                active.updated_at = Set(Utc::now());
                active.update(db).await?;
            }
        }

        Ok(report.is_ready())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use crate::test_utils::setup_test_db;
    use crate::models::module::ActiveModel as ModuleActiveModel;

    fn sample_dates() -> (DateTime<Utc>, DateTime<Utc>) {
        (
            Utc.with_ymd_and_hms(2025, 6, 1, 9, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2025, 6, 30, 17, 0, 0).unwrap(),
        )
    }

    #[tokio::test]
    async fn test_create_assignment() {
        let db = setup_test_db().await;
        let (from, due) = sample_dates();

        let module = ModuleActiveModel {
            code: Set("COS301".to_string()),
            year: Set(2025),
            description: Set(Some("Capstone Project".to_string())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("Failed to insert test module");

        let assignment = Model::create(
            &db,
            module.id,
            "Test Assignment",
            Some("Intro to Testing"),
            AssignmentType::Practical,
            from,
            due,
        )
        .await
        .unwrap();

        assert_eq!(assignment.module_id, module.id);
        assert_eq!(assignment.name, "Test Assignment");
        assert_eq!(assignment.status, Status::Setup); // status defaults to Setup
    }

    #[tokio::test]
    async fn test_edit_assignment() {
        let db = setup_test_db().await;
        let (from, due) = sample_dates();

        let module = ModuleActiveModel {
            code: Set("COS301".to_string()),
            year: Set(2025),
            description: Set(Some("Capstone Project".to_string())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("Failed to insert test module");

        let created = Model::create(
            &db,
            module.id,
            "Initial",
            Some("Initial Desc"),
            AssignmentType::Assignment,
            from,
            due,
        )
        .await
        .unwrap();

        let updated = Model::edit(
            &db,
            created.id,
            module.id,
            "Updated Name",
            Some("Updated Desc"),
            AssignmentType::Practical,
            from,
            due,
        )
        .await
        .unwrap();

        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.status, created.status); // status remains unchanged
    }

    #[tokio::test]
    async fn test_filter_assignments_by_query_and_sort() {
        let db = setup_test_db().await;
        let (from, due) = sample_dates();

        let module = ModuleActiveModel {
            code: Set("COS301".to_string()),
            year: Set(2025),
            description: Set(Some("Capstone Project".to_string())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("Failed to insert test module");

        Model::create(
            &db,
            module.id,
            "Rust Basics",
            Some("Learn Rust"),
            AssignmentType::Assignment,
            from,
            due,
        )
        .await
        .unwrap();
        Model::create(
            &db,
            module.id,
            "Advanced Rust",
            Some("Ownership and lifetimes"),
            AssignmentType::Assignment,
            from,
            due,
        )
        .await
        .unwrap();
        Model::create(
            &db,
            module.id,
            "Python Basics",
            Some("Learn Python"),
            AssignmentType::Assignment,
            from,
            due,
        )
        .await
        .unwrap();

        let rust_results = Model::filter(
            &db,
            module.id.try_into().unwrap(),
            10,
            Some("name".into()),
            Some("rust".into()),
        )
        .await
        .unwrap();

        assert_eq!(rust_results.len(), 2);
        assert!(rust_results
            .iter()
            .all(|a| a.name.to_lowercase().contains("rust")));
    }

}