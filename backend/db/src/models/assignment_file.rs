use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue::Set, DbErr, DatabaseConnection};
use std::fs;
use std::path::PathBuf;
use strum::{Display, EnumIter, EnumString};
use util::execution_config::ExecutionConfig;
use util::paths::{
    storage_root, config_dir, main_dir, makefile_dir, mark_allocator_dir, memo_dir, spec_dir,
};

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

    /// Computes the full directory path using path utilities.
    pub fn full_directory_path(module_id: i64, assignment_id: i64, file_type: &FileType) -> PathBuf {
        match file_type {
            FileType::Config => config_dir(module_id, assignment_id),
            FileType::Main => main_dir(module_id, assignment_id),
            FileType::Memo => memo_dir(module_id, assignment_id),
            FileType::Makefile => makefile_dir(module_id, assignment_id),
            FileType::MarkAllocator => mark_allocator_dir(module_id, assignment_id),
            FileType::Spec => spec_dir(module_id, assignment_id),
        }
    }

    pub fn full_path(&self) -> PathBuf {
        storage_root().join(&self.path)
    }

    /// Saves (or overwrites) a file of a given type under the canonical directory for that type.
    /// For `Config`, also ensures a `config.json` is present at the canonical path.
    pub async fn save_file(
        db: &DatabaseConnection,
        assignment_id: i64,
        module_id: i64,
        file_type: FileType,
        filename: &str,
        bytes: &[u8],
    ) -> Result<Self, DbErr> {
        use crate::models::assignment_file::{Column, Entity as AssignmentFile};
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        use std::ffi::OsStr;

        let now = Utc::now();

        // If a row of this type already exists for this assignment, overwrite the tracked file.
        if let Some(existing) = AssignmentFile::find()
            .filter(Column::AssignmentId.eq(assignment_id))
            .filter(Column::FileType.eq(file_type.clone()))
            .one(db)
            .await?
        {
            let tracked_full = storage_root().join(&existing.path);
            if let Some(parent) = tracked_full.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| DbErr::Custom(format!("Failed to create directory: {e}")))?;
            }
            fs::write(&tracked_full, bytes)
                .map_err(|e| DbErr::Custom(format!("Failed to write file: {e}")))?;

            // Mirror to canonical config.json if needed
            if file_type == FileType::Config {
                let canonical =
                    Self::full_directory_path(module_id, assignment_id, &FileType::Config)
                        .join("config.json");
                if canonical != tracked_full {
                    if let Some(parent) = canonical.parent() {
                        fs::create_dir_all(parent)
                            .map_err(|e| DbErr::Custom(format!("Failed to create directory: {e}")))?;
                    }
                    fs::write(&canonical, bytes)
                        .map_err(|e| DbErr::Custom(format!("Failed to write canonical file: {e}")))?;
                }
            }

            let mut am: ActiveModel = existing.into();
            am.filename = Set(filename.to_string());
            am.updated_at = Set(now);
            return am.update(db).await;
        }

        // No existing row: create directory and choose a canonical stored filename
        let dir_path = Self::full_directory_path(module_id, assignment_id, &file_type);
        fs::create_dir_all(&dir_path)
            .map_err(|e| DbErr::Custom(format!("Failed to create directory: {e}")))?;

        let stored_filename: String = if file_type == FileType::Config {
            // canonical name for configs
            "config.json".to_string()
        } else {
            PathBuf::from(filename)
                .file_name()
                .unwrap_or_else(|| OsStr::new("file"))
                .to_string_lossy()
                .to_string()
        };

        let file_path = dir_path.join(&stored_filename);
        fs::write(&file_path, bytes)
            .map_err(|e| DbErr::Custom(format!("Failed to write file: {e}")))?;

        let relative_path = file_path
            .strip_prefix(storage_root())
            .unwrap()
            .to_string_lossy()
            .to_string();

        let partial = ActiveModel {
            assignment_id: Set(assignment_id),
            filename: Set(stored_filename),
            path: Set(relative_path),
            file_type: Set(file_type),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        partial.insert(db).await
    }

    /// Loads the file contents from disk based on the path stored in the model.
    pub fn load_file(&self) -> Result<Vec<u8>, std::io::Error> {
        let full_path = storage_root().join(&self.path);
        fs::read(full_path)
    }

    /// Deletes the file from disk (but not the DB record).
    pub fn delete_file_only(&self) -> Result<(), std::io::Error> {
        let full_path = storage_root().join(&self.path);
        fs::remove_file(full_path)
    }

    pub async fn get_base_files(
        db: &DatabaseConnection,
        assignment_id: i64,
    ) -> Result<Vec<Self>, DbErr> {
        use crate::models::assignment_file::Column;
        Entity::find()
            .filter(Column::AssignmentId.eq(assignment_id))
            .filter(Column::FileType.eq(FileType::Spec))
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
    use util::paths::storage_root;
    use util::test_helpers::setup_test_storage_root;

    fn fake_bytes() -> Vec<u8> {
        vec![0x50, 0x4B, 0x03, 0x04] // ZIP file signature
    }

    #[tokio::test]
    #[ignore]
    async fn test_save_and_load_file() {
        let _tmp = setup_test_storage_root();
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
        let full_path = storage_root().join(&saved.path);
        assert!(full_path.exists());

        // Load contents
        let bytes = saved.load_file().unwrap();
        assert_eq!(bytes, content);

        // Delete file only
        saved.delete_file_only().unwrap();
        assert!(!full_path.exists());
    }
}
