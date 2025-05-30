use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Represents a file submitted as part of a student's assignment submission.
///
/// Each file is associated with a specific submission and saved on disk.
/// The database stores metadata like the original filename, relative path,
/// and creation/update timestamps.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "submission_files")]
pub struct Model {
    /// Unique identifier of the file.
    #[sea_orm(primary_key)]
    pub id: i64,

    /// Foreign key referencing the associated submission.
    pub submission_id: i64,

    /// The original filename uploaded by the user.
    pub filename: String,

    /// Relative file path from the storage root.
    pub path: String,

    /// Timestamp of when the file was created.
    pub created_at: DateTime<Utc>,

    /// Timestamp of the last update to the file metadata.
    pub updated_at: DateTime<Utc>,
}

/// Describes relationships to other entities in the database.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Defines the relationship to the parent assignment submission.
    #[sea_orm(
        belongs_to = "super::assignment_submission::Entity",
        from = "Column::SubmissionId",
        to = "super::assignment_submission::Column::Id"
    )]
    AssignmentSubmission,
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Returns the root directory used for storing assignment files on disk.
    ///
    /// # Returns
    /// - `PathBuf` pointing to the base directory.
    ///
    /// Uses the `ASSIGNMENT_STORAGE_ROOT` environment variable if set,
    /// otherwise defaults to `data/assignment_files`.
    pub fn storage_root() -> PathBuf {
        env::var("ASSIGNMENT_STORAGE_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/assignment_files"))
    }

    /// Constructs the full directory path for a submission file based on
    /// its module and assignment identifiers.
    ///
    /// # Arguments
    /// - `module_id`: ID of the module containing the assignment.
    /// - `assignment_id`: ID of the specific assignment.
    ///
    /// # Returns
    /// - `PathBuf` with the complete directory path.
    pub fn full_directory_path(module_id: i64, assignment_id: i64) -> PathBuf {
        Self::storage_root()
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{assignment_id}"))
            .join("submission_files")
    }

    /// Computes the absolute path to the stored file on disk.
    ///
    /// # Returns
    /// - `PathBuf` pointing to the file location.
    pub fn full_path(&self) -> PathBuf {
        Self::storage_root().join(&self.path)
    }

    /// Saves a file to disk and creates or updates its metadata in the database.
    ///
    /// This method:
    /// 1. Creates a temporary DB entry.
    /// 2. Looks up the associated assignment and module.
    /// 3. Saves the file with a generated name on disk.
    /// 4. Updates the DB entry with the file path.
    ///
    /// # Arguments
    /// - `db`: Reference to the active database connection.
    /// - `submission_id`: ID of the submission this file belongs to.
    /// - `filename`: The original filename as submitted.
    /// - `bytes`: The file content as a byte slice.
    ///
    /// # Returns
    /// - `Ok(Model)`: The complete, updated `Model` representing the saved file.
    /// - `Err(DbErr)`: If any database or filesystem operation fails.
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

        // Step 4: Write file to disk
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

        // Step 5: Update DB with path
        let mut model: ActiveModel = inserted.into();
        model.path = Set(relative_path);
        model.updated_at = Set(Utc::now());

        model.update(db).await
    }

    /// Loads the contents of the stored file from disk.
    ///
    /// # Returns
    /// - `Ok(Vec<u8>)`: The file contents as bytes.
    /// - `Err(std::io::Error)`: If reading the file fails.
    pub fn load_file(&self) -> Result<Vec<u8>, std::io::Error> {
        fs::read(self.full_path())
    }

    /// Deletes the file from disk without removing the database record.
    ///
    /// # Returns
    /// - `Ok(())`: If the file was successfully deleted.
    /// - `Err(std::io::Error)`: If the file deletion failed.
    pub fn delete_file_only(&self) -> Result<(), std::io::Error> {
        fs::remove_file(self.full_path())
    }
}
#[cfg(test)]
mod tests {
    use crate::models::{assignment::AssignmentType, submission_file::Model};
    use crate::test_utils::setup_test_db;
    use chrono::Utc;
    use sea_orm::{ActiveModelTrait, Set};
    use std::env;
    use tempfile::TempDir;

    fn fake_bytes() -> Vec<u8> {
        vec![0x50, 0x4B, 0x03, 0x04] // ZIP header (PK...)
    }

    fn override_storage_dir(temp: &TempDir) {
        env::set_var("ASSIGNMENT_STORAGE_ROOT", temp.path());
    }

    #[tokio::test]
    async fn test_save_load_delete_submission_file() {
        let temp_dir = TempDir::new().unwrap();
        override_storage_dir(&temp_dir);
        let db = setup_test_db().await;

        // Insert dummy user (needed for submission FK)
        let user = crate::models::user::Model::create(
            &db,
            "12345678",
            "testuser@example.com",
            "password123",
            false,
        )
        .await
        .expect("Failed to insert user");

        // Insert dummy module
        let module = crate::models::module::ActiveModel {
            code: Set("COS301".to_string()),
            year: Set(2025),
            description: Set(Some("Capstone".to_string())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("Failed to insert module");

        // Insert dummy assignment
        let assignment = crate::models::assignment::Model::create(
            &db,
            module.id,
            "Test Assignment",
            Some("Description"),
            AssignmentType::Practical,
            Utc::now(),
            Utc::now(),
        )
        .await
        .expect("Failed to insert assignment");

        // Insert dummy submission with real user_id
        let submission = crate::models::assignment_submission::ActiveModel {
            assignment_id: Set(assignment.id),
            user_id: Set(user.id),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("Failed to insert submission");

        // Save a file
        let content = fake_bytes();
        let filename = "solution.zip";
        let file = Model::save_file(&db, submission.id, filename, &content)
            .await
            .expect("Failed to save file");

        assert_eq!(file.submission_id, submission.id);
        assert_eq!(file.filename, filename);
        assert!(file.path.contains("submission_files"));

        // Check file exists on disk
        let full_path = Model::storage_root().join(&file.path);
        assert!(full_path.exists(), "File not written to disk");

        // Load file content
        let loaded = file.load_file().expect("Failed to load file");
        assert_eq!(loaded, content);

        // Delete file only
        file.delete_file_only().expect("Failed to delete file");
        assert!(!full_path.exists(), "File not deleted from disk");
    }
}
