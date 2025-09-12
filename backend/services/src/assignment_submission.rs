use crate::service::{Service, AppError, ToActiveModel};
use db::{
    models::assignment::{Entity as AssignmentEntity, Column as AssignmentColumn},
    models::assignment_submission::{ActiveModel, Entity as AssignmentSubmissionEntity, Column as AssignmentSubmissionColumn, Model},
    repository::Repository,
};
use util::filters::FilterParam;
use util::execution_config::{ExecutionConfig, execution_config::GradingPolicy};
use sea_orm::{DbErr, Set};
use chrono::Utc;
use std::path::PathBuf;
use std::{fs, env};
use std::collections::HashSet;
use serde_json;

pub use db::models::assignment_submission::Model as AssignmentSubmission;

#[derive(Debug, Clone)]
pub struct CreateAssignmentSubmission {
    pub assignment_id: i64,
    pub user_id: i64,
    pub attempt: i64,
    pub earned: i64,
    pub total: i64,
    pub is_practice: bool,
    pub filename: String,
    pub file_hash: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct UpdateAssignmentSubmission {
    pub id: i64,
    pub ignored: Option<bool>,
}

impl ToActiveModel<AssignmentSubmissionEntity> for CreateAssignmentSubmission {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let now = Utc::now();
        Ok(ActiveModel {
            assignment_id: Set(self.assignment_id),
            user_id: Set(self.user_id),
            attempt: Set(self.attempt),
            is_practice: Set(self.is_practice),
            ignored: Set(false),
            earned: Set(self.earned),
            total: Set(self.total),
            filename: Set(self.filename.to_string()),
            file_hash: Set(self.file_hash.to_string()),
            path: Set("".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
    }
}

impl ToActiveModel<AssignmentSubmissionEntity> for UpdateAssignmentSubmission {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let submission = match Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::find_by_id(self.id).await {
            Ok(Some(submission)) => submission,
            Ok(None) => {
                return Err(AppError::from(DbErr::RecordNotFound(format!("Submission not found for ID {}", self.id))));
            }
            Err(err) => return Err(AppError::from(err)),
        };
        let mut active: ActiveModel = submission.into();

        if let Some(ignored) = self.ignored {
            active.ignored = Set(ignored);
        }

        active.updated_at = Set(Utc::now());

        Ok(active)
    }
}

pub struct AssignmentSubmissionService;

impl<'a> Service<'a, AssignmentSubmissionEntity, AssignmentSubmissionColumn, CreateAssignmentSubmission, UpdateAssignmentSubmission> for AssignmentSubmissionService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓

    fn create(
            params: CreateAssignmentSubmission,
        ) -> std::pin::Pin<Box<dyn std::prelude::rust_2024::Future<Output = Result<<AssignmentSubmissionEntity as sea_orm::EntityTrait>::Model, AppError>> + Send + 'a>> {
        Box::pin(async move {
            if params.earned > params.total {
                return Err(AppError::Database(DbErr::Custom("Earned score cannot be greater than total score".into())));
            }
            
            let inserted: Model = Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::create(params.clone().into_active_model().await?).await?;

            let ext = PathBuf::from(params.filename)
                .extension()
                .map(|e| e.to_string_lossy().to_string());

            let stored_filename = match ext {
                Some(ext) => format!("{}.{}", inserted.id, ext),
                None => inserted.id.to_string(),
            };

            let assignment = Repository::<AssignmentEntity, AssignmentColumn>::find_by_id(params.assignment_id).await?
                .ok_or_else(|| DbErr::RecordNotFound(format!("Assignment ID {} not found", params.assignment_id)))?;
            let dir_path = Self::full_directory_path(assignment.module_id, params.assignment_id, params.user_id, params.attempt);
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

            Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::update(model).await.map_err(AppError::from)
        })
    }
}

impl AssignmentSubmissionService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

    pub fn storage_root() -> PathBuf {
        env::var("ASSIGNMENT_STORAGE_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("data/assignment_files"))
    }

    pub fn full_directory_path(
        module_id: i64,
        assignment_id: i64,
        user_id: i64,
        attempt: i64,
    ) -> PathBuf {
        Self::storage_root()
            .join(format!("module_{module_id}"))
            .join(format!("assignment_{assignment_id}"))
            .join("assignment_submissions")
            .join(format!("user_{user_id}"))
            .join(format!("attempt_{attempt}"))
    }

    pub async fn full_path(
        id: i64
    ) -> Result<PathBuf, DbErr> {
        let submission = Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::find_by_id(id).await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("Submission ID {} not found", id)))?;
        Ok(Self::storage_root().join(submission.path))
    }

    pub async fn load_file(
        id: i64
    ) -> Result<Vec<u8>, std::io::Error> {
        let path = Self::full_path(id).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        fs::read(path)
    }

    pub async fn delete_file_only(
        id: i64
    ) -> Result<(), std::io::Error> {
        let path = Self::full_path(id).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        fs::remove_file(path)
    }

    pub async fn find_by_assignment(
        assignment_id: i64,
    ) -> Result<Vec<i64>, DbErr> {
        let submissions = Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::find_all(
            &vec![
                FilterParam::eq("assignment_id", assignment_id),
            ],
            None
        ).await?;

        Ok(submissions.into_iter().map(|s| s.id as i64).collect())
    }
    
    pub async fn get_latest_submissions_for_assignment(
        assignment_id: i64,
    ) -> Result<Vec<Model>, DbErr> {
        let all = Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::find_all(
            &vec![
                FilterParam::eq("assignment_id", assignment_id),
            ],
            Some("user_id,-attempt".to_string())
        ).await?;

        let mut seen = HashSet::new();
        let mut latest = Vec::new();

        for s in all {
            if seen.insert(s.user_id) {
                latest.push(s);
            }
        }

        Ok(latest)
    }

    pub async fn get_best_for_user(
        assignment_id: i64,
        user_id: i64,
    ) -> Result<Option<Model>, DbErr> {
        let mut subs = Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::find_all(
            &vec![
                FilterParam::eq("assignment_id", assignment_id),
                FilterParam::eq("user_id", user_id),
                FilterParam::eq("ignored", false),
                FilterParam::eq("is_practice", false),
            ],
            None
        ).await?;

        if subs.is_empty() {
            return Ok(None);
        }

        let assignment = Repository::<AssignmentEntity, AssignmentColumn>::find_by_id(assignment_id).await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("Assignment ID {} not found", assignment_id)))?;
        let cfg = assignment
            .config
            .as_ref()
            .and_then(|j| serde_json::from_value::<ExecutionConfig>(j.clone()).ok())
            .unwrap_or_else(ExecutionConfig::default_config);

        match cfg.marking.grading_policy {
            GradingPolicy::Best => {
                subs.sort_by_key(|s| std::cmp::Reverse(s.earned * 1000 / s.total));
                Ok(subs.into_iter().next())
            }
            GradingPolicy::Last => {
                subs.sort_by_key(|s| std::cmp::Reverse(s.created_at));
                Ok(subs.into_iter().next())
            }
        }
    }
}