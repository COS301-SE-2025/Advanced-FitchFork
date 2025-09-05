use crate::service::{Service, ToActiveModel};
use db::{
    models::assignment_file::{Entity, ActiveModel, FileType, Model},
    repositories::{repository::Repository, assignment_file_repository::AssignmentFileRepository},
    filters::AssignmentFileFilter,
};
use sea_orm::{DbErr, Set};
use std::{env, fs, path::PathBuf};
use chrono::{Utc, DateTime};

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
    id: i64,
    pub filename: Option<String>,
}

impl ToActiveModel<Entity> for CreateAssignmentFile {
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
        let now: DateTime<Utc> = Utc::now();
        Ok(ActiveModel {
            assignment_id: Set(self.assignment_id),
            filename: Set(self.filename),
            path: Set("".to_string()), // will be updated after write
            file_type: Set(self.file_type.parse::<FileType>().map_err(|e| DbErr::Custom(format!("Invalid file type: {e}")))?),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
    }
}

impl ToActiveModel<Entity> for UpdateAssignmentFile {
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
        let file = match AssignmentFileRepository::find_by_id(self.id).await {
            Ok(Some(file)) => file,
            Ok(None) => {
                return Err(DbErr::RecordNotFound(format!("File ID {} not found", self.id)));
            }
            Err(err) => return Err(err),
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

impl<'a> Service<'a, Entity, CreateAssignmentFile, UpdateAssignmentFile, AssignmentFileFilter, AssignmentFileRepository> for AssignmentFileService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓

    fn create(
            params: CreateAssignmentFile,
        ) -> std::pin::Pin<Box<dyn std::prelude::rust_2024::Future<Output = Result<<Entity as sea_orm::EntityTrait>::Model, DbErr>> + Send + 'a>> {
        Box::pin(async move {
            if let Some(existing) = AssignmentFileRepository::find_one(
                AssignmentFileFilter {
                    assignment_id: Some(params.assignment_id),
                    file_type: Some(params.file_type.parse::<FileType>().map_err(|e| DbErr::Custom(format!("Invalid file type: {e}")))?.clone()),
                    ..Default::default()
                },
                None,
            )
                .await?
            {
                let existing_path = AssignmentFileService::storage_root().join(&existing.path);
                let _ = fs::remove_file(existing_path); // Silently ignore failure

                AssignmentFileRepository::delete(existing.id).await?;
            }

            let inserted: Model = AssignmentFileRepository::create(params.clone().into_active_model().await?).await?;

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

            AssignmentFileRepository::update(model).await.map_err(DbErr::from)
        })
    }
}

impl AssignmentFileService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

    pub fn storage_root() -> PathBuf {
        env::var("ASSIGNMENT_STORAGE_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/assignment_files"))
    }

    pub fn full_directory_path(module_id: i64, assignment_id: i64, file_type: &FileType) -> PathBuf {
        Self::storage_root()
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{assignment_id}"))
            .join(file_type.to_string())
    }

    pub fn full_path(path: &str) -> PathBuf {
        Self::storage_root().join(path)
    }

    pub fn load_file(model: &Model) -> Result<Vec<u8>, std::io::Error> {
        fs::read(Self::full_path(&model.path))
    }

    pub fn delete_file_only(model: &Model) -> Result<(), std::io::Error> {
        fs::remove_file(Self::full_path(&model.path))
    }

    pub async fn get_base_files(assignment_id: i64) -> Result<Vec<Model>, DbErr> {
        AssignmentFileRepository::find_all(AssignmentFileFilter {
                assignment_id: Some(assignment_id),
                ..Default::default()
            },
            None,
        ).await
    }
}