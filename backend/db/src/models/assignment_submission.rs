use crate::models::assignment;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Represents a user's submission for a specific assignment.
///
/// Each submission is linked to one assignment and one user.
/// Timestamps are included to track when the submission was created and last updated.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "assignment_submissions")]
pub struct Model {
    /// Primary key of the submission.
    #[sea_orm(primary_key)]
    pub id: i64,
    /// ID of the related assignment.
    pub assignment_id: i64,
    /// ID of the user who submitted the assignment.
    pub user_id: i64,
    /// Attempt number
    pub attempt: i64,
    /// The original filename uploaded by the user.
    pub filename: String,
    /// Relative file path from the storage root.
    pub path: String,
    /// Timestamp when the submission was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the submission was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Defines relationships between `assignment_submissions` and other tables.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Link to the related assignment.
    #[sea_orm(
        belongs_to = "super::assignment::Entity",
        from = "Column::AssignmentId",
        to = "super::assignment::Column::Id"
    )]
    Assignment,

    /// Link to the user who submitted the assignment.
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

/// Custom behavior for the active model (currently using default behavior).
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Returns the root directory used for storing assignment submissions on disk.
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

    /// Constructs the full directory path for a submission based on
    /// its module and assignment identifiers.
    ///
    /// # Arguments
    /// - `module_id`: ID of the module containing the assignment.
    /// - `assignment_id`: ID of the specific assignment.
    ///
    /// # Returns
    /// - `PathBuf` with the complete directory path.
    pub fn full_directory_path(
        module_id: i64,
        assignment_id: i64,
        user_id: i64,
        attempt: i64,
    ) -> PathBuf {
        Self::storage_root()
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{assignment_id}"))
            .join("assignment_submissions")
            .join(format!("user_{user_id}"))
            .join(format!("attempt_{attempt}"))
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
    /// - `assignment_id`: ID of the assignment this submission is for.
    /// - `user_id`: ID of the user submitting.
    /// - `attempt`: Attempt number,
    /// - `filename`: The original filename as submitted.
    /// - `bytes`: The file content as a byte slice.
    ///
    /// # Returns
    /// - `Ok(Model)`: The complete, updated `Model` representing the saved file.
    /// - `Err(DbErr)`: If any database or filesystem operation fails.
    pub async fn save_file(
        db: &DatabaseConnection,
        assignment_id: i64,
        user_id: i64,
        attempt: i64,
        filename: &str,
        bytes: &[u8],
    ) -> Result<Self, DbErr> {
        let now = Utc::now();

        // Step 1: Insert placeholder model
        let partial = ActiveModel {
            assignment_id: Set(assignment_id),
            user_id: Set(user_id),
            attempt: Set(attempt),
            filename: Set(filename.to_string()),
            path: Set("".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let inserted: Model = partial.insert(db).await?;

        // Step 2: Lookup module_id
        let assignment = assignment::Entity::find_by_id(assignment_id)
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Assignment not found".into()))?;

        let module_id = assignment.module_id;

        // Step 3: Construct stored filename
        let ext = PathBuf::from(filename)
            .extension()
            .map(|e| e.to_string_lossy().to_string());

        let stored_filename = match ext {
            Some(ext) => format!("{}.{}", inserted.id, ext),
            None => inserted.id.to_string(),
        };

        // Step 4: Write file to disk
        let dir_path = Self::full_directory_path(module_id, assignment_id, user_id, attempt);
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
    use super::Model;
    use crate::models::{assignment::AssignmentType, user::Model as UserModel};
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

        // Create dummy user
        let user = UserModel::create(
            &db,
            "u63963920",
            "testuser12y4f@example.com",
            "password123",
            false,
        )
        .await
        .expect("Failed to insert user");

        // Create dummy module
        let module = crate::models::module::ActiveModel {
            code: Set("COS629".to_string()),
            year: Set(9463),
            description: Set(Some("aqw".to_string())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("Failed to insert module");

        // Create dummy assignment
        let assignment = crate::models::assignment::Model::create(
            &db,
            module.id,
            "Test fsh",
            Some("Description"),
            AssignmentType::Practical,
            Utc::now(),
            Utc::now(),
        )
        .await
        .expect("Failed to insert assignment");

        // Create dummy assignment_submission
        let submission = crate::models::assignment_submission::ActiveModel {
            assignment_id: Set(assignment.id),
            user_id: Set(user.id),
            attempt: Set(1),
            filename: Set("solution.zip".to_string()),
            path: Set("".to_string()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("Failed to insert submission");

        // Save file via submission
        let content = fake_bytes();
        let file = Model::save_file(&db, submission.id, user.id, 6, "solution.zip", &content)
            .await
            .expect("Failed to save file");

        assert_eq!(file.assignment_id, assignment.id);
        assert_eq!(file.user_id, user.id);
        assert!(file.path.contains("assignment_submissions"));

        // Confirm file written
        let full_path = Model::storage_root().join(&file.path);
        assert!(full_path.exists());

        // Load content and verify
        let loaded = file.load_file().expect("Failed to load file");
        assert_eq!(loaded, content);

        // Delete file
        file.delete_file_only().expect("Failed to delete file");
        assert!(!full_path.exists());
    }
}
