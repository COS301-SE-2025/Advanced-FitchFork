//! Entity and business logic for managing assignments.
//!
//! This module defines the `Assignment` model, its relations, and
//! methods for creating, editing, and filtering assignments.

use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DbErr, EntityTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter, QueryOrder, Set, JsonValue, 
};
use chrono::{DateTime, Utc};
use std::{env, fs, path::PathBuf};
use serde::{Serialize, Deserialize};
use strum_macros::{Display, EnumIter, EnumString};

/// Assignment model representing the `assignments` table in the database.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "assignments")]
pub struct Model {
    /// Unique identifier for the assignment.
    #[sea_orm(primary_key)]
    pub id: i64,

    /// Foreign key reference to the associated module.
    pub module_id: i64,

    /// Name/title of the assignment.
    pub name: String,

    /// Optional description providing context or details.
    pub description: Option<String>,

    /// Type/category of the assignment (e.g., "Practical", "Project").
    pub assignment_type: AssignmentType,

    /// Timestamp indicating when the assignment becomes available.
    pub available_from: DateTime<Utc>,

    /// Timestamp representing the assignment's due date.
    pub due_date: DateTime<Utc>,

    /// Optional JSON configuration (e.g., grading rules, language, etc.).
    #[sea_orm(column_type = "Json", nullable)]
    pub config: Option<JsonValue>,

    /// Auto-managed creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Auto-managed update timestamp.
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

impl Model {
    /// Create a new assignment in the database.
    ///
    /// # Arguments
    /// - `db`: The active database connection.
    /// - `module_id`: ID of the associated module.
    /// - `name`: Name/title of the assignment.
    /// - `description`: Optional description.
    /// - `assignment_type`: Type/category of the assignment.
    /// - `available_from`: When the assignment becomes available.
    /// - `due_date`: Deadline for submission.
    ///
    /// # Returns
    /// - `Ok(Self)` on success.
    /// - `Err(DbErr)` on failure.
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
            available_from: Set(available_from),
            due_date: Set(due_date),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        active.insert(db).await
    }

    /// Edit an existing assignment.
    ///
    /// # Arguments
    /// - `db`: The active database connection.
    /// - `id`: ID of the assignment to edit.
    /// - `module_id`: The module ID (used for additional safety).
    /// - `name`: Updated name.
    /// - `description`: Updated description.
    /// - `assignment_type`: Updated assignment type.
    /// - `available_from`: Updated availability date.
    /// - `due_date`: Updated due date.
    ///
    /// # Returns
    /// - `Ok(Self)` with the updated record.
    /// - `Err(DbErr::RecordNotFound)` if assignment doesn't exist.
    /// - `Err(DbErr)` for other DB errors.
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

    /// Delete an assignment and remove its file folder from disk.
    ///
    /// # Arguments
    /// - `db`: Active DB connection
    /// - `id`: Assignment ID to delete
    /// - `module_id`: Must match for safety
    ///
    /// # Returns
    /// - `Ok(())` if successful
    /// - `Err(DbErr)` if not found or delete fails
    pub async fn delete(db: &DatabaseConnection, id: i32, module_id: i32) -> Result<(), DbErr> {
        // Find the assignment
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

        // Delete the assignment
        let active = model.into_active_model();
        active.delete(db).await?;

        // Construct path to assignment directory
        let storage_root = env::var("ASSIGNMENT_STORAGE_ROOT")
            .unwrap_or_else(|_| "data/assignment_files".to_string());

        let assignment_dir = PathBuf::from(storage_root)
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{id}"));

        // Try delete recursively (ignores if doesn't exist)
        if assignment_dir.exists() {
            if let Err(e) = fs::remove_dir_all(&assignment_dir) {
                eprintln!("Warning: Failed to delete assignment directory {:?}: {}", assignment_dir, e);
            }
        }

        Ok(())
    }

    /// Filter and sort assignments with pagination support.
    ///
    /// # Arguments
    /// - `db`: The active database connection.
    /// - `page`: The page number to fetch (1-indexed).
    /// - `per_page`: Number of records per page.
    /// - `sort_by`: Optional sort field (`"name"`, `"due_date"`, or `"available_from"`).
    ///              Use `"-"` prefix for descending order (e.g. `"-name"`).
    /// - `query`: Optional case-insensitive filter applied to `name` and `description`.
    ///
    /// # Returns
    /// - `Ok(Vec<Self>)` containing matched assignments.
    /// - `Err(DbErr)` on failure.
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
}
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use crate::test_utils::setup_test_db;
    use crate::models::module::ActiveModel as ModuleActiveModel;
    use crate::models::assignment::AssignmentType;

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

        let _module = ModuleActiveModel {
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
            1,
            "Test Assignment",
            Some("Intro to Testing"),
            AssignmentType::Practical,
            from,
            due,
        )
        .await
        .unwrap();

        assert_eq!(assignment.module_id, 1);
        assert_eq!(assignment.name, "Test Assignment");
        assert_eq!(assignment.description.as_deref(), Some("Intro to Testing"));
        assert_eq!(assignment.assignment_type, AssignmentType::Practical);
    }

    #[tokio::test]
    async fn test_edit_assignment() {
        let db = setup_test_db().await;
        let (from, due) = sample_dates();

        let _module = ModuleActiveModel {
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
            1,
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
            1,
            "Updated Name",
            Some("Updated Desc"),
            AssignmentType::Practical,
            from,
            due,
        )
        .await
        .unwrap();

        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.description.as_deref(), Some("Updated Desc"));
        assert_eq!(updated.assignment_type, AssignmentType::Practical);
    }

    #[tokio::test]
    async fn test_filter_assignments_by_query_and_sort() {
        let db = setup_test_db().await;
        let (from, due) = sample_dates();

        let _module = ModuleActiveModel {
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

        Model::create(&db, 1, "Rust Basics", Some("Learn Rust"), AssignmentType::Assignment, from, due)
            .await
            .unwrap();
        Model::create(&db, 1, "Advanced Rust", Some("Ownership and lifetimes"), AssignmentType::Assignment, from, due)
            .await
            .unwrap();
        Model::create(&db, 1, "Python Basics", Some("Learn Python"), AssignmentType::Assignment, from, due)
            .await
            .unwrap();

        let rust_results = Model::filter(&db, 1, 10, Some("name".into()), Some("rust".into()))
            .await
            .unwrap();

        assert_eq!(rust_results.len(), 2);
        assert!(rust_results.iter().all(|a| a.name.to_lowercase().contains("rust")));
    }
}
