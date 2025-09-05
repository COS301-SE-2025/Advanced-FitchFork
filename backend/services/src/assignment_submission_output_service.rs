use crate::service::{Service, ToActiveModel};
use db::{
    models::assignment_submission_output::{ActiveModel, Entity, Model},
    repositories::{assignment_submission_output_repository::AssignmentSubmissionOutputRepository, assignment_repository::AssignmentRepository, assignment_task_repository::AssignmentTaskRepository, repository::Repository},
    filters::AssignmentSubmissionOutputFilter,
};
use sea_orm::{DbErr, Set};
use chrono::Utc;
use std::path::PathBuf;
use std::{fs, env};

#[derive(Debug, Clone)]
pub struct CreateAssignmentSubmissionOutput {
    assignment_id: i64,
    task_id: i64,
    filename: String,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct UpdateAssignmentSubmissionOutput {
    id: i64,
    filename: Option<String>,
}

impl ToActiveModel<Entity> for CreateAssignmentSubmissionOutput {
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

impl ToActiveModel<Entity> for UpdateAssignmentSubmissionOutput {
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
        let file = match AssignmentSubmissionOutputRepository::find_by_id(self.id).await {
            Ok(Some(file)) => file,
            Ok(None) => {
                return Err(DbErr::RecordNotFound(format!("File not found for ID {}", self.id)));
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

pub struct AssignmentSubmissionOutputService;

impl<'a> Service<'a, Entity, CreateAssignmentSubmissionOutput, UpdateAssignmentSubmissionOutput, AssignmentSubmissionOutputFilter, AssignmentSubmissionOutputRepository> for AssignmentSubmissionOutputService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓
}

impl AssignmentSubmissionOutputService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓
}