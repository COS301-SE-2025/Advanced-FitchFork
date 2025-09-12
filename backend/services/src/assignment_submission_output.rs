use crate::service::{Service, AppError, ToActiveModel};
use db::{
    models::assignment::{Entity as AssignmentEntity, Column as AssignmentColumn},
    models::assignment_submission::{Entity as AssignmentSubmissionEntity, Column as AssignmentSubmissionColumn},
    models::assignment_submission_output::{ActiveModel, Entity as AssignmentSubmissionOutputEntity, Column as AssignmentSubmissionOutputColumn, Model},
    repository::Repository,
};
use util::filters::FilterParam;
use sea_orm::{DbErr, RuntimeErr, Set};
use chrono::Utc;
use std::path::PathBuf;
use std::{fs, env};

pub use db::models::assignment_submission_output::Model as AssignmentSubmissionOutput;

#[derive(Debug, Clone)]
pub struct CreateAssignmentSubmissionOutput {
    pub task_id: i64,
    pub submission_id: i64,
    pub filename: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct UpdateAssignmentSubmissionOutput {
    pub id: i64,
}

impl ToActiveModel<AssignmentSubmissionOutputEntity> for CreateAssignmentSubmissionOutput {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let now = Utc::now();
        Ok(ActiveModel {
            task_id: Set(self.task_id),
            submission_id: Set(self.submission_id),
            path: Set("".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
    }
}

impl ToActiveModel<AssignmentSubmissionOutputEntity> for UpdateAssignmentSubmissionOutput {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let output = match Repository::<AssignmentSubmissionOutputEntity, AssignmentSubmissionOutputColumn>::find_by_id(self.id).await {
            Ok(Some(output)) => output,
            Ok(None) => {
                return Err(AppError::from(DbErr::RecordNotFound(format!("Submission Output not found for ID {}", self.id))));
            }
            Err(err) => return Err(AppError::from(err)),
        };
        let mut active: ActiveModel = output.into();

        active.updated_at = Set(Utc::now());

        Ok(active)
    }
}

pub struct AssignmentSubmissionOutputService;

impl<'a> Service<'a, AssignmentSubmissionOutputEntity, AssignmentSubmissionOutputColumn, CreateAssignmentSubmissionOutput, UpdateAssignmentSubmissionOutput> for AssignmentSubmissionOutputService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓

    fn create(
            params: CreateAssignmentSubmissionOutput,
        ) -> std::pin::Pin<Box<dyn std::prelude::rust_2024::Future<Output = Result<<AssignmentSubmissionOutputEntity as sea_orm::EntityTrait>::Model, AppError>> + Send + 'a>> {
        Box::pin(async move {
            let inserted: Model = Repository::<AssignmentSubmissionOutputEntity, AssignmentSubmissionOutputColumn>::create(params.clone().into_active_model().await?).await?;

            let ext = PathBuf::from(params.filename)
                .extension()
                .map(|e| e.to_string_lossy().to_string());

            let stored_filename = match ext {
                Some(ext) => format!("{}.{}", inserted.id, ext),
                None => inserted.id.to_string(),
            };

            // Get submission
            let submission = Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::find_by_id(params.submission_id).await?
                .ok_or_else(|| DbErr::RecordNotFound(format!("Submission ID {} not found", params.submission_id)))?;
            let assignment = Repository::<AssignmentEntity, AssignmentColumn>::find_by_id(submission.assignment_id).await?
                .ok_or_else(|| DbErr::RecordNotFound(format!("Assignment ID {} not found", submission.assignment_id)))?;
            let dir_path = Self::full_directory_path(
                assignment.module_id,
                assignment.id,
                submission.user_id,
                submission.attempt,
            );
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

            Repository::<AssignmentSubmissionOutputEntity, AssignmentSubmissionOutputColumn>::update(model).await.map_err(AppError::from)
        })
    }
}

impl AssignmentSubmissionOutputService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

    pub fn storage_root() -> PathBuf {
        let relative_root = env::var("ASSIGNMENT_STORAGE_ROOT")
            .unwrap_or_else(|_| "data/assignment_files".to_string());

        let mut dir = std::env::current_dir().expect("Failed to get current dir");

        while let Some(parent) = dir.parent() {
            if dir.ends_with("backend") {
                return dir.join(relative_root);
            }
            dir = parent.to_path_buf();
        }

        PathBuf::from(relative_root)
    }
    pub fn full_directory_path(
        module_id: i64,
        assignment_id: i64,
        user_id: i64,
        attempt_number: i64,
    ) -> PathBuf {
        Self::storage_root()
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{assignment_id}"))
            .join("assignment_submissions")
            .join(format!("user_{user_id}"))
            .join(format!("attempt_{attempt_number}"))
            .join("submission_output")
    }

    pub async fn full_path(
        id: i64
    ) -> Result<PathBuf, DbErr> {
        let output = Repository::<AssignmentSubmissionOutputEntity, AssignmentSubmissionOutputColumn>::find_by_id(id).await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("Submission Output ID {} not found", id)))?;
        Ok(Self::storage_root().join(output.path))
    }

    pub async fn delete_for_submission(
        id: i64,
    ) -> Result<(), DbErr> {
        let outputs = Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::find_all(
            &vec![
                FilterParam::eq("id", id),
            ],
            None
        ).await?;

        for output in outputs {
            let path = AssignmentSubmissionOutputService::full_path(output.id).await?;
            if path.exists() {
                if let Err(e) = fs::remove_file(&path) {
                    eprintln!("Failed to delete file {:?}: {}", path, e);
                }
            }

            Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::delete_by_id(output.id).await?;
        }

        Ok(())
    }

    pub async fn get_output(
        module_id: i64,
        assignment_id: i64,
        submission_id: i64,
    ) -> Result<Vec<(i64, String)>, DbErr> {
        let submission = Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::find_by_id(submission_id)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("Submission ID {} not found", submission_id)))?;

        let base_dir_path = Self::storage_root()
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{assignment_id}"))
            .join("assignment_submissions")
            .join(format!("user_{}", submission.user_id))
            .join(format!("attempt_{}", submission.attempt))
            .join("submission_output");

        if !base_dir_path.exists() {
            return Err(DbErr::Exec(RuntimeErr::Internal(format!(
                "Submission output directory {:?} does not exist",
                base_dir_path
            ))));
        }

        let mut results = Vec::new();

        let read_dir = fs::read_dir(&base_dir_path)
            .map_err(|e| DbErr::Exec(RuntimeErr::Internal(e.to_string())))?;

        for entry_res in read_dir {
            let entry = entry_res.map_err(|e| DbErr::Exec(RuntimeErr::Internal(e.to_string())))?;
            let file_type = entry.file_type().map_err(|e| DbErr::Exec(RuntimeErr::Internal(e.to_string())))?;
            if file_type.is_file() {
                let file_path = entry.path();
                if let Some(file_name) = file_path.file_stem().and_then(|n| n.to_str()) {
                    if let Ok(output_id) = file_name.parse::<i64>() {
                        let output = Repository::<AssignmentSubmissionOutputEntity, AssignmentSubmissionOutputColumn>::find_by_id(output_id)
                            .await?
                            .ok_or_else(|| {
                                DbErr::RecordNotFound(format!("Submission Output ID {} not found", output_id))
                            })?;

                        let content = fs::read_to_string(&file_path)
                            .map_err(|e| DbErr::Exec(RuntimeErr::Internal(e.to_string())))?;
                        results.push((output.task_id, content));
                    }
                }
            }
        }

        Ok(results)
    }
}