use crate::service::{Service, ToActiveModel};
use db::{
    models::assignment_submission::{ActiveModel, Entity, Model},
    repositories::{assignment_repository::AssignmentRepository, assignment_submission_repository::AssignmentSubmissionRepository, repository::Repository},
    filters::AssignmentSubmissionFilter,
};
use sea_orm::{DbErr, Set};
use chrono::Utc;
use std::path::PathBuf;
use std::{fs, env};

#[derive(Debug, Clone)]
pub struct CreateAssignmentSubmission {
    pub assignment_id: i64,
    pub user_id: i64,
    pub attempt: i64,
    pub is_practice: bool,
    pub filename: String,
    pub file_hash: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct UpdateAssignmentSubmission {
    pub id: i64,
}

impl ToActiveModel<Entity> for CreateAssignmentSubmission {
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
        let now = Utc::now();
        Ok(ActiveModel {
            assignment_id: Set(self.assignment_id),
            user_id: Set(self.user_id),
            attempt: Set(self.attempt),
            is_practice: Set(self.is_practice),
            filename: Set(self.filename.to_string()),
            file_hash: Set(self.file_hash.to_string()),
            path: Set("".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
    }
}

impl ToActiveModel<Entity> for UpdateAssignmentSubmission {
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
        let submission = match AssignmentSubmissionRepository::find_by_id(self.id).await {
            Ok(Some(submission)) => submission,
            Ok(None) => {
                return Err(DbErr::RecordNotFound(format!("Submission not found for ID {}", self.id)));
            }
            Err(err) => return Err(err),
        };
        let mut active: ActiveModel = submission.into();

        active.updated_at = Set(Utc::now());

        Ok(active)
    }
}

pub struct AssignmentSubmissionService;

impl<'a> Service<'a, Entity, CreateAssignmentSubmission, UpdateAssignmentSubmission, AssignmentSubmissionFilter, AssignmentSubmissionRepository> for AssignmentSubmissionService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓

    fn create(
            params: CreateAssignmentSubmission,
        ) -> std::pin::Pin<Box<dyn std::prelude::rust_2024::Future<Output = Result<<Entity as sea_orm::EntityTrait>::Model, DbErr>> + Send + 'a>> {
        Box::pin(async move {
            let inserted: Model = AssignmentSubmissionRepository::create(params.clone().into_active_model().await?).await?;

            let ext = PathBuf::from(params.filename)
                .extension()
                .map(|e| e.to_string_lossy().to_string());

            let stored_filename = match ext {
                Some(ext) => format!("{}.{}", inserted.id, ext),
                None => inserted.id.to_string(),
            };

            let assignment = AssignmentRepository::find_by_id(params.assignment_id).await?
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

            AssignmentSubmissionRepository::update(model).await
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
        let submission = AssignmentSubmissionRepository::find_by_id(id).await?
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
        let submissions = AssignmentSubmissionRepository::find_all(
            AssignmentSubmissionFilter {
                assignment_id: Some(assignment_id),
                ..Default::default()
            },
            None,
        ).await?;

        Ok(submissions.into_iter().map(|s| s.id as i64).collect())
    }
    
    pub async fn get_latest_submissions_for_users(
        assignment_id: i64,
        user_ids: Vec<i64>,
    ) -> Result<Vec<Model>, DbErr> {
        let mut latest_submissions = Vec::new();

        for user_id in user_ids {
            let latest_submission = AssignmentSubmissionRepository::find_one(
                AssignmentSubmissionFilter {
                    assignment_id: Some(assignment_id),
                    user_id: Some(user_id),
                    ..Default::default()
                },
                Some("-attempt".to_string()),
            ).await?;

            if let Some(submission) = latest_submission {
                latest_submissions.push(submission);
            }
        }

        Ok(latest_submissions)
    }
}