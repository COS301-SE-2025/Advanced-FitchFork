// models/assignment_file.rs

use chrono::{DateTime, Utc};
// use code_runner::ExecutionConfig;
use sea_orm::{ActiveValue::Set, DbErr, DatabaseConnection};
use sea_orm::entity::prelude::*;
use std::env;
use std::fs;
use std::path::PathBuf;
use strum::{Display, EnumIter, EnumString};
use util::execution_config::ExecutionConfig;

/// Represents a file associated with an assignment, such as a spec, main file, memo, or submission.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "assignment_files")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    /// Foreign key reference to an assignment.
    pub assignment_id: i64,

    /// Original file name.
    pub filename: String,

    /// Relative path to the stored file from the storage root.
    pub path: String,

    /// Type of the file (spec, main, memo, submission).
    pub file_type: FileType,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Enum representing the type/category of an assignment file.
#[derive(Debug, Clone, PartialEq, EnumIter, EnumString, Display, DeriveActiveEnum)]
#[strum(ascii_case_insensitive)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "assignment_file_type"
)]
pub enum FileType {
    #[strum(serialize = "spec")]
    #[sea_orm(string_value = "spec")]
    Spec,
    #[strum(serialize = "main")]
    #[sea_orm(string_value = "main")]
    Main,
    #[strum(serialize = "memo")]
    #[sea_orm(string_value = "memo")]
    Memo,
    #[strum(serialize = "makefile")]
    #[sea_orm(string_value = "makefile")]
    Makefile,
    #[strum(serialize = "mark_allocator")]
    #[sea_orm(string_value = "mark_allocator")]
    MarkAllocator,
    #[strum(serialize = "config")]
    #[sea_orm(string_value = "config")]
    Config,
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
    /// Loads and returns the `ExecutionConfig` if the file type is `Config`.
    /// Requires `module_id` because it's not stored in the DB.
    pub fn load_execution_config(&self, module_id: i64) -> Result<ExecutionConfig, String> {
        if self.file_type != FileType::Config {
            return Err("File is not of type 'config'".to_string());
        }

        ExecutionConfig::get_execution_config(module_id, self.assignment_id)
    }

    /// Returns the base directory for assignment file storage from the environment.
    pub fn storage_root() -> PathBuf {
        env::var("ASSIGNMENT_STORAGE_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/assignment_files"))
    }

    /// Computes the full directory path based on module ID, assignment ID, and file type.
    pub fn full_directory_path(
        module_id: i64,
        assignment_id: i64,
        file_type: &FileType,
    ) -> PathBuf {
        Self::storage_root()
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{assignment_id}"))
            .join(file_type.to_string())
    }

    pub fn full_path(&self) -> PathBuf {
        Self::storage_root().join(&self.path)
    }

    pub async fn save_file(
        db: &DatabaseConnection,
        assignment_id: i64,
        module_id: i64,
        file_type: FileType,
        filename: &str,
        bytes: &[u8],
    ) -> Result<Self, DbErr> {
        let now = Utc::now();

        use sea_orm::ColumnTrait;
        use sea_orm::EntityTrait;
        use sea_orm::QueryFilter;

        use crate::models::assignment_file::{Column, Entity as AssignmentFile};

        if let Some(existing) = AssignmentFile::find()
            .filter(Column::AssignmentId.eq(assignment_id))
            .filter(Column::FileType.eq(file_type.clone()))
            .one(db)
            .await?
        {
            let existing_path = Self::storage_root().join(&existing.path);
            let _ = fs::remove_file(existing_path); // Silently ignore failure

            existing.delete(db).await?;
        }

        let partial = ActiveModel {
            assignment_id: Set(assignment_id),
            filename: Set(filename.to_string()),
            path: Set("".to_string()), // will be updated after write
            file_type: Set(file_type.clone()),
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

        let dir_path = Self::full_directory_path(module_id, assignment_id, &file_type);
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
    
    pub async fn get_base_files(
        db: &DatabaseConnection,
        assignment_id: i64,
    ) -> Result<Vec<Self>, DbErr> {
        Entity::find()
            .filter(Column::AssignmentId.eq(assignment_id))
            .filter(Column::FileType.eq(FileType::Main))
            .all(db)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::assignment::AssignmentType;
    use crate::test_utils::setup_test_db;
    use chrono::Utc;
    use sea_orm::Set;
    use std::env;
    use tempfile::TempDir;

    fn fake_bytes() -> Vec<u8> {
        vec![0x50, 0x4B, 0x03, 0x04] // ZIP file signature
    }

    fn override_storage_dir(temp: &TempDir) {
        unsafe {
            env::set_var("ASSIGNMENT_STORAGE_ROOT", temp.path());
        }
    }

    #[tokio::test]
    #[ignore]
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
            AssignmentType::Practical,
            Utc::now(),
            Utc::now(),
        )
        .await
        .expect("Insert assignment failed");

        let content = fake_bytes();
        let filename = "test_file.zip";
        let saved = Model::save_file(
            &db,
            1, // assignment_id
            1, // module_id
            FileType::Spec,
            filename,
            &content,
        )
        .await
        .unwrap();

        assert_eq!(saved.assignment_id, 1);
        assert_eq!(saved.filename, filename);
        assert_eq!(saved.file_type, FileType::Spec);

        // Confirm file on disk
        let full_path = Model::storage_root().join(&saved.path);
        //The error was this line
        assert!(full_path.exists());

        // Load contents
        let bytes = saved.load_file().unwrap();
        assert_eq!(bytes, content);

        // Delete file only
        saved.delete_file_only().unwrap();
        assert!(!full_path.exists());
    }
}
