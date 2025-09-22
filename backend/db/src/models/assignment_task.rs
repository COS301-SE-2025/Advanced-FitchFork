use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::EntityTrait;
use strum::EnumIter;

/// Assignment task model representing the `assignment_tasks` table.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "assignment_tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub assignment_id: i64,
    pub task_number: i64,
    pub name: String,
    pub command: String,
    pub code_coverage: bool,
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

impl Model {
    /// Create a new task in the database.
    pub async fn create(
        db: &DatabaseConnection,
        assignment_id: i64,
        task_number: i64,
        name: &str,
        command: &str,
        code_coverage: bool,
    ) -> Result<Self, DbErr> {
        let active = ActiveModel {
            assignment_id: Set(assignment_id),
            task_number: Set(task_number),
            name: Set(name.to_string()),
            command: Set(command.to_string()),
            code_coverage: Set(code_coverage),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };
        active.insert(db).await
    }

    /// Get a task by its ID.
    pub async fn get_by_id(db: &DatabaseConnection, id: i64) -> Result<Option<Self>, DbErr> {
        Entity::find_by_id(id).one(db).await
    }

    /// Get all tasks for a specific assignment.
    pub async fn get_by_assignment_id(
        db: &DatabaseConnection,
        assignment_id: i64,
    ) -> Result<Vec<Self>, DbErr> {
        Entity::find()
            .filter(Column::AssignmentId.eq(assignment_id))
            .all(db)
            .await
    }

    pub async fn edit(
        db: &DatabaseConnection,
        id: i64,
        name: Option<&str>,
        command: Option<&str>,
        code_coverage: Option<bool>,
    ) -> Result<Self, DbErr> {
        let Some(task) = Self::get_by_id(db, id).await? else {
            return Err(DbErr::RecordNotFound("Task not found".into()));
        };

        let mut active = task.into_active_model();
        if let Some(n) = name {
            active.name = Set(n.to_string());
        }
        if let Some(c) = command {
            active.command = Set(c.to_string());
        }
        if let Some(cc) = code_coverage {
            active.code_coverage = Set(cc);
        }
        active.updated_at = Set(Utc::now());
        active.update(db).await
    }

    /// Back-compat: reuse `edit` to update name + command only.
    pub async fn edit_command_and_name(
        db: &DatabaseConnection,
        id: i64,
        new_name: &str,
        new_command: &str,
    ) -> Result<Self, DbErr> {
        Self::edit(db, id, Some(new_name), Some(new_command), None).await
    }

    /// Delete a task by ID.
    pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<(), DbErr> {
        if let Some(task) = Self::get_by_id(db, id).await? {
            task.delete(db).await.map(|_| ())
        } else {
            Err(DbErr::RecordNotFound("Task not found".into()))
        }
    }
}
