use crate::service::{Service, ToActiveModel};
use db::{
    models::assignment_overwrite_file::{ActiveModel, Entity, Model},
    repositories::{assignment_overwrite_file_repository::AssignmentOverwriteFileRepository, assignment_repository::AssignmentRepository, assignment_task_repository::AssignmentTaskRepository, repository::Repository},
    filters::AssignmentOverwriteFileFilter,
};
use sea_orm::{DbErr, Set};
use chrono::Utc;
use std::path::PathBuf;
use std::{fs, env};

#[derive(Debug, Clone)]
pub struct CreateAssignmentOverwriteFile {
    pub assignment_id: i64,
    pub task_id: i64,
    pub filename: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct UpdateAssignmentOverwriteFile {
    pub id: i64,
    pub filename: Option<String>,
}

impl ToActiveModel<Entity> for CreateAssignmentOverwriteFile {
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
        let now = Utc::now();
        Ok(ActiveModel {
            assignment_id: Set(self.assignment_id),
            task_id: Set(self.task_id),
            filename: Set(self.filename),
            path: Set("".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
    }
}

impl ToActiveModel<Entity> for UpdateAssignmentOverwriteFile {
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
        let file = match AssignmentOverwriteFileRepository::find_by_id(self.id).await {
            Ok(Some(file)) => file,
            Ok(None) => {
                return Err(DbErr::RecordNotFound(format!("Overwrite File not found for ID {}", self.id)));
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

pub struct AssignmentOverwriteFileService;

impl<'a> Service<'a, Entity, CreateAssignmentOverwriteFile, UpdateAssignmentOverwriteFile, AssignmentOverwriteFileFilter, AssignmentOverwriteFileRepository> for AssignmentOverwriteFileService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓

    fn create(
            params: CreateAssignmentOverwriteFile,
        ) -> std::pin::Pin<Box<dyn std::prelude::rust_2024::Future<Output = Result<<Entity as sea_orm::EntityTrait>::Model, DbErr>> + Send + 'a>> {
        Box::pin(async move {
            let inserted: Model = AssignmentOverwriteFileRepository::create(params.clone().into_active_model().await?).await?;

            let ext = PathBuf::from(params.filename)
                .extension()
                .map(|e| e.to_string_lossy().to_string());

            let stored_filename = match ext {
                Some(ext) => format!("{}.{}", inserted.id, ext),
                None => inserted.id.to_string(),
            };

            let assignment = AssignmentRepository::find_by_id(params.assignment_id).await?
                .ok_or_else(|| DbErr::RecordNotFound(format!("Assignment ID {} not found", params.assignment_id)))?;
            let task = AssignmentTaskRepository::find_by_id(params.task_id).await?
                .ok_or_else(|| DbErr::RecordNotFound(format!("Task ID {} not found", params.task_id)))?;
            let dir_path = Self::full_directory_path(assignment.module_id, params.assignment_id, task.task_number);
            fs::create_dir_all(&dir_path)
                .map_err(|e| DbErr::Custom(format!("Failed to create directory: {e}")))?;

            let file_path = dir_path.join(&stored_filename);
            let relative_path = file_path
                .strip_prefix(Self::storage_root())
                .unwrap()
                .to_string_lossy()
                .to_string();

            fs::write(&file_path, params.bytes)
                .map_err(|e| DbErr::Custom(format!("Failed to write file: {e}")))?;

            let mut model: ActiveModel = inserted.into();
            model.path = Set(relative_path);
            model.updated_at = Set(Utc::now());

            AssignmentOverwriteFileRepository::update(model).await
        })
    }
}

impl AssignmentOverwriteFileService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

    pub fn storage_root() -> PathBuf {
        env::var("ASSIGNMENT_STORAGE_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/assignment_files"))
    }

    pub fn full_directory_path(
        module_id: i64,
        assignment_id: i64,
        task_number: i64
    ) -> PathBuf {
        Self::storage_root()
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{assignment_id}"))
            .join("overwrite_files")
            .join(format!("task_{task_number}"))
    }

    pub async fn full_path(
        id: i64
    ) -> Result<PathBuf, DbErr> {
        let overwrite = AssignmentOverwriteFileRepository::find_by_id(id).await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("Overwrite File ID {} not found", id)))?;
        Ok(Self::storage_root().join(overwrite.path))
    }

    pub async fn load_file(
        id: i64,
    ) -> Result<Vec<u8>, std::io::Error> {
        let overwrite = AssignmentOverwriteFileRepository::find_by_id(id)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("DB error: {e}")))?
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, format!("Overwrite File ID {} not found", id)))?;
        let full_path = Self::storage_root().join(overwrite.path);
        fs::read(full_path)
    }

    pub async fn delete_file_only(
        id: i64,
    ) -> Result<(), std::io::Error> {
        let overwrite = AssignmentOverwriteFileRepository::find_by_id(id)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("DB error: {e}")))?
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, format!("Overwrite File ID {} not found", id)))?;
        let full_path = Self::storage_root().join(overwrite.path);
        fs::remove_file(full_path)
    }
}