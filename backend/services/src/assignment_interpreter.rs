use crate::service::{Service, AppError, ToActiveModel};
use db::{
    models::assignment_interpreter::{ActiveModel, Entity, Column, Model},
    repository::Repository,
};
use util::filters::FilterParam;
use sea_orm::{DbErr, Set};
use chrono::Utc;
use std::{env, fs};
use std::path::PathBuf;

pub use db::models::assignment_interpreter::Model as AssignmentInterpreter;

#[derive(Debug, Clone)]
pub struct CreateAssignmentInterpreter {
    pub assignment_id: i64,
    pub module_id: i64,
    pub filename: String,
    pub command: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct UpdateAssignmentInterpreter {
    pub id: i64,
}

impl ToActiveModel<Entity> for CreateAssignmentInterpreter {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let now = Utc::now();
        Ok(ActiveModel {
            assignment_id: Set(self.assignment_id),
            filename: Set(self.filename.to_string()),
            path: Set("".to_string()), // updated after file write
            command: Set(self.command.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
    }
}

impl ToActiveModel<Entity> for UpdateAssignmentInterpreter {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let announcement = match Repository::<Entity, Column>::find_by_id(self.id).await {
            Ok(Some(announcement)) => announcement,
            Ok(None) => {
                return Err(AppError::from(DbErr::RecordNotFound(format!("AssignmentInterpreter not found for ID {}", self.id))));
            }
            Err(err) => return Err(AppError::from(err)),
        };
        let mut active: ActiveModel = announcement.into();

        active.updated_at = Set(Utc::now());

        Ok(active)
    }
}

pub struct AssignmentInterpreterService;

impl<'a> Service<'a, Entity, Column, CreateAssignmentInterpreter, UpdateAssignmentInterpreter> for AssignmentInterpreterService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓

    fn create(
            params: CreateAssignmentInterpreter,
        ) -> std::pin::Pin<Box<dyn std::prelude::rust_2024::Future<Output = Result<<Entity as sea_orm::EntityTrait>::Model, AppError>> + Send + 'a>> {
        Box::pin(async move {
            if let Some(existing) = Repository::<Entity, Column>::find_one(
                &vec![
                    FilterParam::eq("assignment_id", params.assignment_id),
                    FilterParam::eq("filename", params.clone().filename),
                ],
                &vec![],
                None,
            ).await? {
                let existing_path = AssignmentInterpreterService::storage_root().join(&existing.path);
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

            let dir_path = Self::full_directory_path(params.module_id, params.assignment_id);
            fs::create_dir_all(&dir_path)
                .map_err(|e| sea_orm::DbErr::Custom(format!("Failed to create directory: {}", e)))?;

            let file_path = dir_path.join(&stored_filename);
            let relative_path = file_path
                .strip_prefix(Self::storage_root())
                .unwrap()
                .to_string_lossy()
                .to_string();

            fs::write(&file_path, params.bytes)
                .map_err(|e| sea_orm::DbErr::Custom(format!("Failed to write file: {}", e)))?;

            let mut model: ActiveModel = inserted.into();
            model.path = Set(relative_path);
            model.updated_at = Set(Utc::now());

            Repository::<Entity, Column>::update(model).await.map_err(AppError::from)
        })
    }
}

impl AssignmentInterpreterService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

    pub fn storage_root() -> PathBuf {
        env::var("ASSIGNMENT_STORAGE_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/interpreters"))
    }

    pub fn full_directory_path(
        module_id: i64,
        assignment_id: i64,
    ) -> PathBuf {
        Self::storage_root()
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id))
            .join("interpreter")
    }

    pub async fn load_file(
        id: i64,
    ) -> Result<Vec<u8>, std::io::Error> {
        let interpreter = Repository::<Entity, Column>::find_by_id(id)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("DB error: {e}")))?
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, format!("Interpreter ID {} not found", id)))?;
        let full_path = Self::storage_root().join(interpreter.path);
        fs::read(full_path)
    }

    pub async fn delete_file_only(
        id: i64,
    ) -> Result<(), std::io::Error> {
        let interpreter = Repository::<Entity, Column>::find_by_id(id)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("DB error: {e}")))?
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, format!("Interpreter ID {} not found", id)))?;
        let full_path = Self::storage_root().join(interpreter.path);
        fs::remove_file(full_path)
    }
}