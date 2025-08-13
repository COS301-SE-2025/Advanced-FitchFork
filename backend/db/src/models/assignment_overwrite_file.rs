use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Represents a file used to overwrite specific parts of an assignment during evaluation.
/// Includes metadata such as its related assignment, task, filename, and storage path.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "assignment_overwrite_files")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub assignment_id: i64,
    pub task_id: i64,
    pub filename: String,
    pub path: String,
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

    #[sea_orm(
        belongs_to = "super::assignment_task::Entity",
        from = "Column::TaskId",
        to = "super::assignment_task::Column::TaskNumber"
    )]
    AssignmentTask,
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn storage_root() -> PathBuf {
        env::var("ASSIGNMENT_STORAGE_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/assignment_files"))
    }

    pub fn full_directory_path(module_id: i64, assignment_id: i64, task_number: i64) -> PathBuf {
        Self::storage_root()
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{assignment_id}"))
            .join("overwrite_files")
            .join(format!("task_{task_number}"))
    }

    pub fn full_path(&self) -> PathBuf {
        Self::storage_root().join(&self.path)
    }

    pub async fn save_file(
        db: &DatabaseConnection,
        assignment_id: i64,
        task_id: i64,
        filename: &str,
        bytes: &[u8],
    ) -> Result<Self, DbErr> {
        let now = Utc::now();

        let partial = ActiveModel {
            assignment_id: Set(assignment_id),
            task_id: Set(task_id),
            filename: Set(filename.to_string()),
            path: Set("".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let inserted: Model = partial.insert(db).await?;

        let ext = PathBuf::from(filename)
            .extension()
            .map(|e| e.to_string_lossy().to_string());

        let stored_filename = match ext {
            Some(ext) => format!("{}.{}", inserted.id, ext),
            None => inserted.id.to_string(),
        };

        let assignment = super::assignment::Entity::find_by_id(assignment_id)
            .one(db)
            .await
            .map_err(|e| DbErr::Custom(format!("DB error finding assignment: {}", e)))?
            .ok_or_else(|| DbErr::Custom("Assignment not found".to_string()))?;

        let module_id = assignment.module_id;

        let task = super::assignment_task::Entity::find_by_id(task_id)
            .one(db)
            .await
            .map_err(|e| DbErr::Custom(format!("DB error finding task: {}", e)))?
            .ok_or_else(|| DbErr::Custom("Task not found".to_string()))?;

        let task_number = task.task_number;

        let dir_path = Self::full_directory_path(module_id, assignment_id, task_number);
        fs::create_dir_all(&dir_path)
            .map_err(|e| DbErr::Custom(format!("Failed to create directory: {e}")))?;

        let file_path = dir_path.join(&stored_filename);
        let relative_path = file_path
            .strip_prefix(Self::storage_root())
            .unwrap()
            .to_string_lossy()
            .to_string();

        fs::write(&file_path, bytes)
            .map_err(|e| DbErr::Custom(format!("Failed to write file: {e}")))?;

        let mut model: ActiveModel = inserted.into();
        model.path = Set(relative_path);
        model.updated_at = Set(Utc::now());

        model.update(db).await
    }

    /// Loads the file contents from disk based on the path stored in the model.
    pub fn load_file(&self) -> Result<Vec<u8>, std::io::Error> {
        let full_path = Self::storage_root().join(&self.path);
        fs::read(full_path)
    }

    /// Deletes the file from disk (but not the DB record).
    pub fn delete_file_only(&self) -> Result<(), std::io::Error> {
        let full_path = Self::storage_root().join(&self.path);
        fs::remove_file(full_path)
    }
}