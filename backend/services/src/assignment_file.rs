use crate::service::{AppError, Service, ToActiveModel};
use chrono::{DateTime, Utc};
use db::{
    models::assignment_file::{ActiveModel, Column, Entity, Model},
    repository::Repository,
};
use sea_orm::{DbErr, Set};
use std::ffi::OsStr;
use std::{fs, path::PathBuf};
use util::execution_config::ExecutionConfig;
use util::filters::FilterParam;
use util::paths::{
    config_dir, main_dir, makefile_dir, mark_allocator_dir, memo_dir, spec_dir, storage_root,
};

pub use db::models::assignment_file::FileType;
pub use db::models::assignment_file::Model as AssignmentFile;

#[derive(Debug, Clone)]
pub struct CreateAssignmentFile {
    pub assignment_id: i64,
    pub module_id: i64,
    pub file_type: FileType,
    pub filename: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct UpdateAssignmentFile {
    pub id: i64,
    pub filename: Option<String>,
}

impl ToActiveModel<Entity> for CreateAssignmentFile {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let now: DateTime<Utc> = Utc::now();
        Ok(ActiveModel {
            assignment_id: Set(self.assignment_id),
            filename: Set(self.filename),
            path: Set("".to_string()), // will be updated after write
            file_type: Set(self.file_type),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
    }
}

impl ToActiveModel<Entity> for UpdateAssignmentFile {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let file = match Repository::<Entity, Column>::find_by_id(self.id).await {
            Ok(Some(file)) => file,
            Ok(None) => {
                return Err(AppError::from(DbErr::RecordNotFound(format!(
                    "File ID {} not found",
                    self.id
                ))));
            }
            Err(err) => return Err(AppError::from(err)),
        };

        let mut active: ActiveModel = file.into();

        if let Some(filename) = self.filename {
            active.filename = Set(filename);
        }

        active.updated_at = Set(Utc::now());

        Ok(active)
    }
}

pub struct AssignmentFileService;

impl<'a> Service<'a, Entity, Column, CreateAssignmentFile, UpdateAssignmentFile>
    for AssignmentFileService
{
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓

    fn create(
        params: CreateAssignmentFile,
    ) -> std::pin::Pin<
        Box<
            dyn std::prelude::rust_2024::Future<
                    Output = Result<<Entity as sea_orm::EntityTrait>::Model, AppError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            let now = Utc::now();

            // If a row of this type already exists for this assignment, overwrite the tracked file.
            if let Some(existing) = Repository::<Entity, Column>::find_one(
                &vec![
                    FilterParam::eq("assignment_id", params.assignment_id),
                    FilterParam::eq("file_type", params.file_type),
                ],
                &vec![],
                None,
            )
            .await?
            {
                let tracked_full = storage_root().join(&existing.path);
                if let Some(parent) = tracked_full.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| DbErr::Custom(format!("Failed to create directory: {e}")))?;
                }
                fs::write(&tracked_full, params.bytes)
                    .map_err(|e| DbErr::Custom(format!("Failed to write file: {e}")))?;

                // Mirror to canonical config.json if needed
                if params.file_type == FileType::Config {
                    let canonical = Self::full_directory_path(
                        params.module_id,
                        params.assignment_id,
                        &FileType::Config,
                    )
                    .join("config.json");
                    if canonical != tracked_full {
                        if let Some(parent) = canonical.parent() {
                            fs::create_dir_all(parent).map_err(|e| {
                                DbErr::Custom(format!("Failed to create directory: {e}"))
                            })?;
                        }
                        fs::write(&canonical, params.bytes).map_err(|e| {
                            DbErr::Custom(format!("Failed to write canonical file: {e}"))
                        })?;
                    }
                }

                let mut am: ActiveModel = existing.into();
                am.filename = Set(params.filename.to_string());
                am.updated_at = Set(now);

                Repository::<Entity, Column>::update(am).await
            }

            // No existing row: create directory and choose a canonical stored filename
            let dir_path = Self::full_directory_path(
                params.module_id,
                params.assignment_id,
                &params.file_type,
            );
            fs::create_dir_all(&dir_path)
                .map_err(|e| DbErr::Custom(format!("Failed to create directory: {e}")))?;

            let stored_filename: String = if params.file_type == FileType::Config {
                // canonical name for configs
                "config.json".to_string()
            } else {
                PathBuf::from(params.filename)
                    .file_name()
                    .unwrap_or_else(|| OsStr::new("file"))
                    .to_string_lossy()
                    .to_string()
            };

            let file_path = dir_path.join(&stored_filename);
            fs::write(&file_path, params.bytes)
                .map_err(|e| DbErr::Custom(format!("Failed to write file: {e}")))?;

            // In some test envs the temp storage root may differ; fall back to absolute.
            let relative_path = match file_path.strip_prefix(storage_root()) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(_) => file_path.to_string_lossy().to_string(),
            };

            let partial = ActiveModel {
                assignment_id: Set(params.assignment_id),
                filename: Set(stored_filename),
                path: Set(relative_path),
                file_type: Set(params.file_type),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            };

            Repository::<Entity, Column>::insert(partial).await
        })
    }
}

impl AssignmentFileService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

    pub async fn load_execution_config(
        module_id: i64,
        assignment_id: i64,
    ) -> Result<ExecutionConfig, String> {
        let file = match Repository::<Entity, Column>::find_by_id(assignment_id)
            .await
            .map_err(|e| format!("DB error while fetching config file: {:?}", e))?
        {
            Some(f) => f,
            None => return Err("No configuration file found".to_string()),
        };

        if file.file_type != FileType::Config {
            return Err("File is not of type 'config'".to_string());
        }

        ExecutionConfig::get_execution_config(module_id, assignment_id)
    }

    pub fn full_directory_path(
        module_id: i64,
        assignment_id: i64,
        file_type: &FileType,
    ) -> PathBuf {
        match file_type {
            FileType::Config => config_dir(module_id, assignment_id),
            FileType::Main => main_dir(module_id, assignment_id),
            FileType::Memo => memo_dir(module_id, assignment_id),
            FileType::Makefile => makefile_dir(module_id, assignment_id),
            FileType::MarkAllocator => mark_allocator_dir(module_id, assignment_id),
            FileType::Spec => spec_dir(module_id, assignment_id),
        }
    }

    pub fn full_path(path: &str) -> PathBuf {
        storage_root().join(path)
    }

    pub async fn load_file(id: i64) -> Result<Vec<u8>, std::io::Error> {
        let file = Repository::<Entity, Column>::find_by_id(id)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("DB error: {e}")))?
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("File ID {} not found", id),
                )
            })?;
        let full_path = storage_root().join(file.path);
        fs::read(full_path)
    }

    pub async fn delete_file_only(id: i64) -> Result<(), std::io::Error> {
        let file = Repository::<Entity, Column>::find_by_id(id)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("DB error: {e}")))?
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("File ID {} not found", id),
                )
            })?;
        let full_path = storage_root().join(file.path);
        fs::remove_file(full_path)
    }

    pub async fn get_base_files(assignment_id: i64) -> Result<Vec<Model>, DbErr> {
        Repository::<Entity, Column>::find_all(
            &vec![
                FilterParam::eq("assignment_id", assignment_id),
                FilterParam::eq("file_type", FileType::Spec.to_string()),
            ],
            &vec![],
            None,
        )
        .await
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::models::assignment::AssignmentType;
//     use crate::test_utils::setup_test_db;
//     use chrono::Utc;
//     use sea_orm::Set;
//     use util::paths::storage_root;
//     use util::test_helpers::setup_test_storage_root;

//     fn fake_bytes() -> Vec<u8> {
//         vec![0x50, 0x4B, 0x03, 0x04] // ZIP file signature
//     }

//     #[tokio::test]
//     #[ignore]
//     async fn test_save_and_load_file() {
//         let _tmp = setup_test_storage_root();
//         let db = setup_test_db().await;

//         // Insert dummy module so assignment FK passes
//         let _module = crate::models::module::ActiveModel {
//             code: Set("COS301".to_string()),
//             year: Set(2025),
//             description: Set(Some("Capstone".to_string())),
//             created_at: Set(Utc::now()),
//             updated_at: Set(Utc::now()),
//             ..Default::default()
//         }
//         .insert(&db)
//         .await
//         .expect("Insert module failed");

//         // Insert dummy assignment using enum value for assignment_type
//         let _assignment = crate::models::assignment::Model::create(
//             &db,
//             1,
//             "Test Assignment",
//             Some("Desc"),
//             AssignmentType::Practical,
//             Utc::now(),
//             Utc::now(),
//         )
//         .await
//         .expect("Insert assignment failed");

//         let content = fake_bytes();
//         let filename = "test_file.zip";
//         let saved = Model::save_file(
//             &db,
//             1, // assignment_id
//             1, // module_id
//             FileType::Spec,
//             filename,
//             &content,
//         )
//         .await
//         .unwrap();

//         assert_eq!(saved.assignment_id, 1);
//         assert_eq!(saved.filename, filename);
//         assert_eq!(saved.file_type, FileType::Spec);

//         // Confirm file on disk
//         let full_path = storage_root().join(&saved.path);
//         assert!(full_path.exists());

//         // Load contents
//         let bytes = saved.load_file().unwrap();
//         assert_eq!(bytes, content);

//         // Delete file only
//         saved.delete_file_only().unwrap();
//         assert!(!full_path.exists());
//     }
// }
