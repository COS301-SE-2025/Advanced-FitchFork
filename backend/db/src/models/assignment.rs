//! Entity and business logic for managing assignments.
//!
//! This module defines the `Assignment` model, its relations, and
//! methods for creating, editing, and filtering assignments.

use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DbErr, EntityTrait, IntoActiveModel,
    JsonValue, PaginatorTrait, QueryFilter, QueryOrder, Set,
};
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

impl Model {
    pub async fn create(
        db: &DatabaseConnection,
        module_id: i64,
        name: &str,
        description: Option<&str>,
        assignment_type: AssignmentType,
        available_from: DateTime<Utc>,
        due_date: DateTime<Utc>,
        status: Option<Status>,
    ) -> Result<Self, DbErr> {
        let active = ActiveModel {
            module_id: Set(module_id),
            name: Set(name.to_string()),
            description: Set(description.map(|d| d.to_string())),
            assignment_type: Set(assignment_type),
            status: Set(status.unwrap_or(Status::Setup)),
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
        status: Status,
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
        assignment.status = Set(status);
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
            Some(Status::Setup),
        )
        .await
        .unwrap();

        assert_eq!(assignment.module_id, 1);
        assert_eq!(assignment.name, "Test Assignment");
        assert_eq!(assignment.status, Status::Setup);
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
            Some(Status::Setup),
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
            Status::Ready,
        )
        .await
        .unwrap();

        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.status, Status::Ready);
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

        Model::create(&db, 1, "Rust Basics", Some("Learn Rust"), AssignmentType::Assignment, from, due, Some(Status::Open))
            .await
            .unwrap();
        Model::create(&db, 1, "Advanced Rust", Some("Ownership and lifetimes"), AssignmentType::Assignment, from, due, Some(Status::Ready))
            .await
            .unwrap();
        Model::create(&db, 1, "Python Basics", Some("Learn Python"), AssignmentType::Assignment, from, due, Some(Status::Closed))
            .await
            .unwrap();

        let rust_results = Model::filter(&db, 1, 10, Some("name".into()), Some("rust".into()))
            .await
            .unwrap();

        assert_eq!(rust_results.len(), 2);
        assert!(rust_results.iter().all(|a| a.name.to_lowercase().contains("rust")));
    }
}