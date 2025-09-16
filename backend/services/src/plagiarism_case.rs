use crate::service::{Service, AppError, ToActiveModel};
use db::{
    models::plagiarism_case::{ActiveModel, Entity, Column},
    repository::Repository,
};
use sea_orm::{DbErr, Set};
use chrono::Utc;

pub use db::models::plagiarism_case::Model as PlagiarismCase;
pub use db::models::plagiarism_case::Status;

#[derive(Debug, Clone)]
pub struct CreatePlagiarismCase {
    pub assignment_id: i64,
    pub submission_id_1: i64,
    pub submission_id_2: i64,
    pub description: String,
    pub similarity: f32,
}

#[derive(Debug, Clone)]
pub struct UpdatePlagiarismCase {
    pub id: i64,
    pub description: Option<String>,
    pub status: Option<Status>,
    pub similarity: Option<f32>,
}

impl ToActiveModel<Entity> for CreatePlagiarismCase {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let now = Utc::now();
        Ok(ActiveModel {
            assignment_id: Set(self.assignment_id),
            submission_id_1: Set(self.submission_id_1),
            submission_id_2: Set(self.submission_id_2),
            description: Set(self.description.to_string()),
            status: Set(Status::Review),
            similarity: Set(self.similarity),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
    }
}

impl ToActiveModel<Entity> for UpdatePlagiarismCase {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let case = match Repository::<Entity, Column>::find_by_id(self.id).await {
            Ok(Some(case)) => case,
            Ok(None) => {
                return Err(AppError::from(DbErr::RecordNotFound(format!("Case not found for ID {}", self.id))));
            }
            Err(err) => return Err(AppError::from(err)),
        };
        let mut active: ActiveModel = case.into();

        if let Some(status) = self.status {
            active.status = Set(status);
        }

        active.updated_at = Set(Utc::now());

        Ok(active)
    }
}

pub struct PlagiarismCaseService;

impl<'a> Service<'a, Entity, Column, CreatePlagiarismCase, UpdatePlagiarismCase> for PlagiarismCaseService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓
}

impl PlagiarismCaseService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓
}