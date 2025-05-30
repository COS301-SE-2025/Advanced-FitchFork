use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Represents a file submitted by a student as part of an assignment submission.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "submission_files")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    pub submission_id: i64,
    pub filename: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::assignment_submission::Entity",
        from = "Column::SubmissionId",
        to = "super::assignment_submission::Column::Id"
    )]
    AssignmentSubmission,
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn storage_root() -> PathBuf {
        env::var("ASSIGNMENT_STORAGE_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/assignment_files"))
    }

    /// Computes the full directory path based on module ID and assignment ID.
    pub fn full_directory_path(module_id: i64, assignment_id: i64) -> PathBuf {
        Self::storage_root()
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{assignment_id}"))
            .join("submission_files")
    }

    pub fn full_path(&self) -> PathBuf {
        Self::storage_root().join(&self.path)
    }

    pub async fn save_file(
        db: &DatabaseConnection,
        submission_id: i64,
        filename: &str,
        bytes: &[u8],
    ) -> Result<Self, DbErr> {
        let now = Utc::now();

        // Step 1: Insert placeholder model
        let partial = ActiveModel {
            submission_id: Set(submission_id),
            filename: Set(filename.to_string()),
            path: Set("".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let inserted: Model = partial.insert(db).await?;

        // Step 2: Join to get assignment_id and module_id
        use super::assignment;
        use super::assignment_submission;

        let submission = assignment_submission::Entity::find_by_id(submission_id)
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Submission not found".into()))?;

        let assignment = assignment::Entity::find_by_id(submission.assignment_id)
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Assignment not found".into()))?;

        let module_id = assignment.module_id;
        let assignment_id = assignment.id;

        // Step 3: Construct stored filename
        let ext = PathBuf::from(filename)
            .extension()
            .map(|e| e.to_string_lossy().to_string());

        let stored_filename = match ext {
            Some(ext) => format!("{}.{}", inserted.id, ext),
            None => inserted.id.to_string(),
        };

        // Step 4: Determine directory
        let dir_path = Self::full_directory_path(module_id, assignment_id);
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

        // Step 5: Update path in DB
        let mut model: ActiveModel = inserted.into();
        model.path = Set(relative_path);
        model.updated_at = Set(Utc::now());

        model.update(db).await
    }

    pub fn load_file(&self) -> Result<Vec<u8>, std::io::Error> {
        fs::read(self.full_path())
    }

    pub fn delete_file_only(&self) -> Result<(), std::io::Error> {
        fs::remove_file(self.full_path())
    }
}
