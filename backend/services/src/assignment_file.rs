use crate::service::{Service, AppError, ToActiveModel};
use db::{
    models::assignment_file::{Entity, Column, ActiveModel, Model},
    repository::Repository,
};
use util::filters::FilterParam;
use util::execution_config::ExecutionConfig;
use sea_orm::{DbErr, Set};
use std::{env, fs, path::PathBuf};
use chrono::{Utc, DateTime};

pub use db::models::assignment_file::Model as AssignmentFile;
pub use db::models::assignment_file::FileType;

#[derive(Debug, Clone)]
pub struct CreateAssignmentFile {
    pub assignment_id: i64,
    pub module_id: i64,
    pub file_type: String,
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
            file_type: Set(self.file_type.trim().parse::<FileType>().map_err(|e| DbErr::Custom(format!("Invalid file type '{}': {}", self.file_type, e)))?),
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
                return Err(AppError::from(DbErr::RecordNotFound(format!("File ID {} not found", self.id))));
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

impl<'a> Service<'a, Entity, Column, CreateAssignmentFile, UpdateAssignmentFile> for AssignmentFileService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓

    fn create(
            params: CreateAssignmentFile,
        ) -> std::pin::Pin<Box<dyn std::prelude::rust_2024::Future<Output = Result<<Entity as sea_orm::EntityTrait>::Model, AppError>> + Send + 'a>> {
        Box::pin(async move {
            if let Some(existing) = Repository::<Entity, Column>::find_one(
                &vec![
                    FilterParam::eq("assignment_id", params.assignment_id),
                    FilterParam::eq("file_type", params.clone().file_type),
                ],
                &vec![],
                None,
            ).await?
            {
                let existing_path = AssignmentFileService::storage_root().join(&existing.path);
                let _ = fs::remove_file(existing_path); // Silently ignore failure

                Repository::<Entity, Column>::delete_by_id(existing.id).await?;
            }

            let inserted: Model = Repository::<Entity, Column>::create(params.clone().into_active_model().await?).await?;

            let ext = PathBuf::from(params.filename)
                .extension()
                .map(|e| e.to_string_lossy().to_string());

            let stored_filename = match ext {
                Some(ext) => format!("{}.{}", inserted.id, ext),
                None => inserted.id.to_string(),
            };

            let file_type = params.file_type.parse::<FileType>().map_err(|e| DbErr::Custom(format!("Invalid file type: {e}")))?;
            let dir_path =  AssignmentFileService::full_directory_path(params.module_id, params.assignment_id, &file_type);
            fs::create_dir_all(&dir_path)
                .map_err(|e| DbErr::Custom(format!("Failed to create directory: {e}")))?;

            let file_path = dir_path.join(&stored_filename);
            let relative_path = file_path
                .strip_prefix( AssignmentFileService::storage_root())
                .unwrap()
                .to_string_lossy()
                .to_string();

            fs::write(&file_path, params.bytes)
                .map_err(|e| DbErr::Custom(format!("Failed to write file: {e}")))?;

            let mut model: ActiveModel = inserted.into();
            model.path = Set(relative_path);
            model.updated_at = Set(Utc::now());

            Repository::<Entity, Column>::update(model).await.map_err(AppError::from)
        })
    }
}

impl AssignmentFileService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

    pub async fn load_execution_config(module_id: i64, assignment_id: i64) -> Result<ExecutionConfig, String> {
        let file = match Repository::<Entity, Column>::find_by_id(assignment_id).await.map_err(|e| format!("DB error while fetching config file: {:?}", e))? {
            Some(f) => f,
            None => return Err("No configuration file found".to_string()),
        };

        if file.file_type != FileType::Config {
            return Err("File is not of type 'config'".to_string());
        }

        ExecutionConfig::get_execution_config(module_id, assignment_id)
    }

    pub fn storage_root() -> PathBuf {
        env::var("ASSIGNMENT_STORAGE_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/assignment_files"))
    }

    pub fn full_directory_path(
        module_id: i64,
        assignment_id: i64,
        file_type: &FileType
    ) -> PathBuf {
        Self::storage_root()
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{assignment_id}"))
            .join(file_type.to_string())
    }

    pub fn full_path(
        path: &str
    ) -> PathBuf {
        Self::storage_root().join(path)
    }

    pub async fn load_file(
        id: i64,
    ) -> Result<Vec<u8>, std::io::Error> {
        let file = Repository::<Entity, Column>::find_by_id(id)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("DB error: {e}")))?
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, format!("File ID {} not found", id)))?;
        let full_path = Self::storage_root().join(file.path);
        fs::read(full_path)
    }

    pub async fn delete_file_only(
        id: i64,
    ) -> Result<(), std::io::Error> {
        let file = Repository::<Entity, Column>::find_by_id(id)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("DB error: {e}")))?
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, format!("File ID {} not found", id)))?;
        let full_path = Self::storage_root().join(file.path);
        fs::remove_file(full_path)
    }

    // TODO: Change this to get the skeleton files instead of the memo files
    pub async fn get_base_files(
        assignment_id: i64
    ) -> Result<Vec<Model>, DbErr> {
        Repository::<Entity, Column>::find_all(&vec![
                FilterParam::eq("assignment_id", assignment_id),
            ],
            &vec![],
            None,
        ).await
    }
}