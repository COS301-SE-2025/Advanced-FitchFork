use crate::service::{Service, ToActiveModel};
use db::{
    models::assignment_memo_output::{ActiveModel, Entity, Model},
    repositories::{assignment_memo_output_repository::AssignmentMemoOutputRepository, assignment_repository::AssignmentRepository, repository::Repository},
    filters::AssignmentMemoOutputFilter,
};
use sea_orm::{DbErr, Set};
use chrono::Utc;
use std::path::PathBuf;
use std::{env, fs};

#[derive(Debug, Clone)]
pub struct CreateAssignmentMemoOutput {
    pub assignment_id: i64,
    pub task_id: i64,
    pub filename: String,
    pub bytes: Vec<u8>,
}

// Note: Nothing can currently be edited for memo outputs
#[derive(Debug, Clone)]
pub struct UpdateAssignmentMemoOutput {
    pub id: i64,
}

impl ToActiveModel<Entity> for CreateAssignmentMemoOutput {
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
        let now = Utc::now();
        Ok(ActiveModel {
            assignment_id: Set(self.assignment_id),
            task_id: Set(self.task_id),
            path: Set("".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
    }
}

impl ToActiveModel<Entity> for UpdateAssignmentMemoOutput {
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
        let task = match AssignmentMemoOutputRepository::find_by_id(self.id).await {
            Ok(Some(task)) => task,
            Ok(None) => {
                return Err(DbErr::RecordNotFound(format!("Task ID {} not found", self.id)));
            }
            Err(err) => return Err(err),
        };

        let mut active: ActiveModel = task.into();

        active.updated_at = Set(Utc::now());

        Ok(active)
    }
}

pub struct AssignmentMemoOutputService;

impl<'a> Service<'a, Entity, CreateAssignmentMemoOutput, UpdateAssignmentMemoOutput, AssignmentMemoOutputFilter, AssignmentMemoOutputRepository> for AssignmentMemoOutputService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓

    fn create(
            params: CreateAssignmentMemoOutput,
        ) -> std::pin::Pin<Box<dyn std::prelude::rust_2024::Future<Output = Result<<Entity as sea_orm::EntityTrait>::Model, DbErr>> + Send + 'a>> {
        Box::pin(async move {
            let inserted: Model = AssignmentMemoOutputRepository::create(params.clone().into_active_model().await?).await?;

            let ext = PathBuf::from(params.filename)
                .extension()
                .map(|e| e.to_string_lossy().to_string());

            let stored_filename = match ext {
                Some(ext) => format!("{}.{}", inserted.id, ext),
                None => inserted.id.to_string(),
            };

            //Get assignment
            let assignment = AssignmentRepository::find_by_id(params.assignment_id).await?
                .ok_or_else(|| DbErr::RecordNotFound(format!("Assignment ID {} not found", params.assignment_id)))?;
            let dir_path = Self::full_directory_path(assignment.module_id, params.assignment_id);
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

            AssignmentMemoOutputRepository::update(model).await.map_err(DbErr::from)
        })
    }
}

impl AssignmentMemoOutputService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

    pub fn storage_root() -> PathBuf {
        let relative_root = env::var("ASSIGNMENT_STORAGE_ROOT")
            .unwrap_or_else(|_| "data/assignment_files".to_string());
        
        let project_root = env::current_dir().expect("Failed to get current dir");
        
        project_root.join(relative_root)
    }

    pub fn full_directory_path(
        module_id: i64,
        assignment_id: i64
    ) -> PathBuf {
        Self::storage_root()
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{assignment_id}"))
            .join(format!("memo_output"))
    }

    pub async fn full_path(
        id: i64
    ) -> Result<PathBuf, DbErr> {
        let memo_output = AssignmentMemoOutputRepository::find_by_id(id).await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("Memo Output ID {} not found", id)))?;
        Ok(Self::storage_root().join(&memo_output.path))
    }

    // pub fn read_memo_output_file(
    //     module_id: i64,
    //     assignment_id: i64,
    //     file_id: i64,
    // ) -> Result<Vec<u8>, std::io::Error> {
    //     let storage_root = Self::storage_root();

    //     let dir_path = storage_root
    //         .join(format!("module_{module_id}"))
    //         .join(format!("assignment_{assignment_id}"))
    //         .join("memo_output");

    //     let file_path = dir_path.join(file_id.to_string());

    //     std::fs::read(file_path)
    // }
}