use crate::service::{Service, ToActiveModel};
use db::{
    models::plagiarism_case::{ActiveModel, Entity, Status},
    repositories::{plagiarism_case_repository::PlagiarismCaseRepository, repository::Repository},
};
use sea_orm::{DbErr, Set};
use chrono::Utc;

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
}

impl ToActiveModel<Entity> for CreatePlagiarismCase {
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
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
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
        let case = match PlagiarismCaseRepository::find_by_id(self.id).await {
            Ok(Some(case)) => case,
            Ok(None) => {
                return Err(DbErr::RecordNotFound(format!("Case not found for ID {}", self.id)));
            }
            Err(err) => return Err(err),
        };
        let mut active: ActiveModel = case.into();

        active.updated_at = Set(Utc::now());

        Ok(active)
    }
}

pub struct PlagiarismCaseService;

impl<'a> Service<'a, Entity, CreatePlagiarismCase, UpdatePlagiarismCase, PlagiarismCaseRepository> for PlagiarismCaseService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓
}

impl PlagiarismCaseService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓
}