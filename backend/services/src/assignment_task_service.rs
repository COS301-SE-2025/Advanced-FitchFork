use crate::service::{Service, ToActiveModel};
use db::{
    models::assignment_task::{Entity, ActiveModel},
    repositories::{repository::Repository, assignment_task_repository::AssignmentTaskRepository},
    filters::AssignmentTaskFilter,
};
use sea_orm::{DbErr, Set};
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct CreateAssignmentTask {
    pub assignment_id: i64,
    pub task_number: i64,
    pub name: String,
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct UpdateAssignmentTask {
    id: i64,
    pub name: Option<String>,
    pub command: Option<String>,
}

impl ToActiveModel<Entity> for CreateAssignmentTask {
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
        Ok(ActiveModel {
            assignment_id: Set(self.assignment_id),
            task_number: Set(self.task_number),
            name: Set(self.name),
            command: Set(self.command),
            ..Default::default()
        })
    }
}

impl ToActiveModel<Entity> for UpdateAssignmentTask {
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
        let task = match AssignmentTaskRepository::find_by_id(self.id).await {
            Ok(Some(task)) => task,
            Ok(None) => {
                return Err(DbErr::RecordNotFound(format!("Task ID {} not found", self.id)));
            }
            Err(err) => return Err(err),
        };

        let mut active: ActiveModel = task.into();

        if let Some(name) = self.name {
            active.name = Set(name);
        }

        if let Some(command) = self.command {
            active.command = Set(command);
        }

        active.updated_at = Set(Utc::now());

        Ok(active)
    }
}

pub struct AssignmentTaskService;

impl<'a> Service<'a, Entity, CreateAssignmentTask, UpdateAssignmentTask, AssignmentTaskFilter, AssignmentTaskRepository> for AssignmentTaskService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓
}

impl AssignmentTaskService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

    pub async fn tasks_present(assignment_id: i64) -> bool {
        AssignmentTaskRepository::exists(AssignmentTaskFilter {
            assignment_id: Some(assignment_id),
            ..Default::default()
        }).await.unwrap_or(false)
    }
}