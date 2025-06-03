use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, QueryFilter, Set,
};
use strum_macros::EnumIter;

/// Assignment task model representing the `assignment_tasks` table.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "assignment_tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub assignment_id: i64,
    pub task_number: i64,
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

impl Model {
    /// Create a new task in the database.
    pub async fn create(
        db: &DatabaseConnection,
        assignment_id: i64,
        task_number: i64,
        command: &str,
    ) -> Result<Self, DbErr> {
        let active = ActiveModel {
            assignment_id: Set(assignment_id),
            task_number: Set(task_number),
            command: Set(command.to_string()),
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

    /// Edit a task's command.
    pub async fn edit_command(
        db: &DatabaseConnection,
        id: i64,
        new_command: &str,
    ) -> Result<Self, DbErr> {
        if let Some(task) = Self::get_by_id(db, id).await? {
            let mut active = task.into_active_model();
            active.command = Set(new_command.to_string());
            active.updated_at = Set(Utc::now());
            active.update(db).await
        } else {
            Err(DbErr::RecordNotFound("Task not found".into()))
        }
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
