// models/assignment_interpreter.rs

use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Represents an interpreter file associated with an assignment,
/// including the command used to run it.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "interpreters")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    /// Foreign key reference to an assignment.
    pub assignment_id: i64,

    /// Original file name.
    pub filename: String,

    /// Relative path to the stored file from the storage root.
    pub path: String,

    /// Command to run this interpreter.
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
    /// Returns the base directory for interpreter file storage from the environment.
    pub fn storage_root() -> PathBuf {
        env::var("ASSIGNMENT_STORAGE_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/interpreters"))
    }

    /// Computes the full directory path based on module ID and assignment ID.
    pub fn full_directory_path(module_id: i64, assignment_id: i64) -> PathBuf {
        Self::storage_root()
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id))
            .join("interpreter")
    }

    pub async fn save_file(
        db: &DatabaseConnection,
        assignment_id: i64,
        module_id: i64,
        filename: &str,
        command: &str,
        bytes: &[u8],
    ) -> Result<Self, sea_orm::DbErr> {
        use crate::models::assignment_interpreter::{Column, Entity as AssignmentInterpreter};
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        let existing = AssignmentInterpreter::find()
            .filter(Column::AssignmentId.eq(assignment_id))
            .all(db)
            .await?;

        for record in existing {
            let existing_path = Self::storage_root().join(&record.path);
            let _ = fs::remove_file(existing_path);
            record.delete(db).await?;
        }

        let now = Utc::now();

        let partial = ActiveModel {
            assignment_id: Set(assignment_id),
            filename: Set(filename.to_string()),
            path: Set("".to_string()),
            command: Set(command.to_string()),
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

        let dir_path = Self::full_directory_path(module_id, assignment_id);
        fs::create_dir_all(&dir_path)
            .map_err(|e| sea_orm::DbErr::Custom(format!("Failed to create directory: {}", e)))?;

        let file_path = dir_path.join(&stored_filename);
        let relative_path = file_path
            .strip_prefix(Self::storage_root())
            .unwrap()
            .to_string_lossy()
            .to_string();

        fs::write(&file_path, bytes)
            .map_err(|e| sea_orm::DbErr::Custom(format!("Failed to write file: {}", e)))?;

        let mut model: ActiveModel = inserted.into();
        model.path = Set(relative_path);
        model.updated_at = Set(Utc::now());

        model.update(db).await
    }

    /// Load interpreter file content from disk.
    pub fn load_file(&self) -> Result<Vec<u8>, std::io::Error> {
        let full_path = Self::storage_root().join(&self.path);
        fs::read(full_path)
    }

    /// Delete the interpreter file from disk (but not DB record).
    pub fn delete_file_only(&self) -> Result<(), std::io::Error> {
        let full_path = Self::storage_root().join(&self.path);
        fs::remove_file(full_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db;
    use chrono::Utc;
    use sea_orm::Set;
    use tempfile::TempDir;

    fn fake_bytes() -> Vec<u8> {
        vec![0x50, 0x4B, 0x03, 0x04] // ZIP file signature
    }

    fn override_storage_dir(temp: &TempDir) {
        unsafe {
            std::env::set_var("ASSIGNMENT_STORAGE_ROOT", temp.path());
        }
    }

    #[tokio::test]
    async fn test_save_and_load_file() {
        let temp_dir = TempDir::new().unwrap();
        override_storage_dir(&temp_dir);
        let db = setup_test_db().await;

        // Insert dummy module so assignment FK passes
        let _module = crate::models::module::ActiveModel {
            code: Set("COS301".to_string()),
            year: Set(2025),
            description: Set(Some("Capstone".to_string())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("Insert module failed");

        // Insert dummy assignment using enum value for assignment_type
        let _assignment = crate::models::assignment::Model::create(
            &db,
            1,
            "Test Assignment",
            Some("Desc"),
            crate::models::assignment::AssignmentType::Practical,
            Utc::now(),
            Utc::now(),
        )
        .await
        .expect("Insert assignment failed");

        let content = fake_bytes();
        let filename = "interpreter.sh";
        let command = "sh interpreter.sh";

        let saved = Model::save_file(&db, 1, 1, filename, command, &content)
            .await
            .expect("Failed to save interpreter");

        assert_eq!(saved.assignment_id, 1);
        assert_eq!(saved.filename, filename);
        assert_eq!(saved.command, command);

        // Confirm file on disk
        let full_path = Model::storage_root().join(&saved.path);
        assert!(full_path.exists());

        // Load contents
        let bytes = saved.load_file().unwrap();
        assert_eq!(bytes, content);

        // Delete file only
        saved.delete_file_only().unwrap();
        assert!(!full_path.exists());
    }
}
