use crate::service::{AppError, Service, ToActiveModel};
use chrono::Utc;
use db::{
    models::assignment::{Column as AssignmentColumn, Entity as AssignmentEntity},
    models::assignment_submission::{
        Column as AssignmentSubmissionColumn, Entity as AssignmentSubmissionEntity,
    },
    models::assignment_submission_output::{
        ActiveModel, Column as AssignmentSubmissionOutputColumn,
        Entity as AssignmentSubmissionOutputEntity, Model,
    },
    repository::Repository,
};
use sea_orm::{DbErr, RuntimeErr, Set};
use std::path::PathBuf;
use std::fs;
use util::filters::FilterParam;
use util::paths::{ensure_dir, storage_root, submission_output_dir};

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
        let output = match Repository::<
            AssignmentSubmissionOutputEntity,
            AssignmentSubmissionOutputColumn,
        >::find_by_id(self.id)
        .await
        {
            Ok(Some(output)) => output,
            Ok(None) => {
                return Err(AppError::from(DbErr::RecordNotFound(format!(
                    "Submission Output not found for ID {}",
                    self.id
                ))));
            }
            Err(err) => return Err(AppError::from(err)),
        };
        let mut active: ActiveModel = output.into();

        active.updated_at = Set(Utc::now());

        Ok(active)
    }
}

pub struct AssignmentSubmissionOutputService;

impl<'a>
    Service<
        'a,
        AssignmentSubmissionOutputEntity,
        AssignmentSubmissionOutputColumn,
        CreateAssignmentSubmissionOutput,
        UpdateAssignmentSubmissionOutput,
    > for AssignmentSubmissionOutputService
{
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓

    fn create(
        params: CreateAssignmentSubmissionOutput,
    ) -> std::pin::Pin<
        Box<
            dyn std::prelude::rust_2024::Future<
                    Output = Result<
                        <AssignmentSubmissionOutputEntity as sea_orm::EntityTrait>::Model,
                        AppError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            let inserted: Model = Repository::<
                AssignmentSubmissionOutputEntity,
                AssignmentSubmissionOutputColumn,
            >::create(params.clone().into_active_model().await?)
            .await?;

            let ext = PathBuf::from(params.filename)
                .extension()
                .map(|e| e.to_string_lossy().to_string());

            let stored_filename = match ext {
                Some(ext) => format!("{}.{}", inserted.id, ext),
                None => inserted.id.to_string(),
            };

            // Get submission
            let submission =
                Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::find_by_id(
                    params.submission_id,
                )
                .await?
                .ok_or_else(|| {
                    DbErr::RecordNotFound(format!(
                        "Submission ID {} not found",
                        params.submission_id
                    ))
                })?;
            let assignment = Repository::<AssignmentEntity, AssignmentColumn>::find_by_id(
                submission.assignment_id,
            )
            .await?
            .ok_or_else(|| {
                DbErr::RecordNotFound(format!(
                    "Assignment ID {} not found",
                    submission.assignment_id
                ))
            })?;
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

    pub async fn full_path(id: i64) -> Result<PathBuf, DbErr> {
        let output = Repository::<AssignmentSubmissionOutputEntity, AssignmentSubmissionOutputColumn>::find_by_id(id).await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("Submission Output ID {} not found", id)))?;
        Ok(storage_root().join(output.path))
    }

    pub async fn delete_for_submission(id: i64) -> Result<(), DbErr> {
        let outputs =
            Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::find_all(
                &vec![FilterParam::eq("id", id)],
                &vec![],
                None,
            )
            .await?;

        for output in outputs {
            let path = AssignmentSubmissionOutputService::full_path(output.id).await?;
            if path.exists() {
                if let Err(e) = fs::remove_file(&path) {
                    eprintln!("Failed to delete file {:?}: {}", path, e);
                }
            }

            Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::delete_by_id(
                output.id,
            )
            .await?;
        }

        Ok(())
    }

    pub async fn get_output(
        module_id: i64,
        assignment_id: i64,
        submission_id: i64,
    ) -> Result<Vec<(i64, String)>, DbErr> {
        let submission =
            Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::find_by_id(
                submission_id,
            )
            .await?
            .ok_or_else(|| {
                DbErr::RecordNotFound(format!("Submission ID {} not found", submission_id))
            })?;

        let base_dir_path = submission_output_dir(
            module_id,
            assignment_id,
            submission.user_id,
            submission.attempt,
        );

        if !base_dir_path.exists() {
            return Err(DbErr::Exec(RuntimeErr::Internal(format!(
                "Submission output directory {:?} does not exist",
                base_dir_path
            ))));
        }

        let mut results = Vec::new();
        for entry in fs::read_dir(&base_dir_path).map_err(|e| DbErr::Exec(RuntimeErr::Internal(e.to_string())))? {
            let entry = entry.map_err(|e| DbErr::Exec(RuntimeErr::Internal(e.to_string())))?;
            if entry.file_type().is_file() {
                let file_path = entry.path();
                if let Some(stem) = file_path.file_stem().and_then(|n| n.to_str()) {
                    if let Ok(output_id) = stem.parse::<i64>() {
                        if let Some(output) = Repository::<
                            AssignmentSubmissionOutputEntity,
                            AssignmentSubmissionOutputColumn,
                        >::find_by_id(output_id)
                        .await?
                        .ok_or_else(|| {
                            DbErr::RecordNotFound(format!(
                                "Submission Output ID {} not found",
                                output_id
                            ))
                        })
                        {
                            let content = fs::read_to_string(&file_path).map_err(|e| DbErr::Exec(RuntimeErr::Internal(e.to_string())))?;
                            results.push((output.task_id, content));
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}
