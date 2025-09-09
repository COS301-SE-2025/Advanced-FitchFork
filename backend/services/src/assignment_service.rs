use crate::service::{Service, AppError, ToActiveModel};
use crate::assignment_task_service::AssignmentTaskService;
use crate::assignment_file_service::AssignmentFileService;
use db::{
    models::{assignment::{Entity, ActiveModel, AssignmentType, Status, ReadinessReport}, assignment_file::FileType},
    repositories::{repository::Repository, assignment_repository::AssignmentRepository},
};
use sea_orm::{DbErr, Set, IntoActiveModel};
use chrono::{DateTime, Utc};
use std::{env, fs, path::PathBuf};
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub struct CreateAssignment {
    pub module_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub assignment_type: String,
    pub available_from: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UpdateAssignment {
    pub id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub assignment_type: Option<String>,
    pub available_from: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
}

impl ToActiveModel<Entity> for CreateAssignment {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        validate_dates(self.available_from, self.due_date)?;
        let now = Utc::now();
        Ok(ActiveModel {
            module_id: Set(self.module_id),
            name: Set(self.name),
            description: Set(self.description.map(|d| d)),
            assignment_type: Set(self.assignment_type.parse::<AssignmentType>().map_err(|_| DbErr::Custom("Invalid assignment_type".into()))?),
            status: Set(Status::Setup),
            available_from: Set(self.available_from),
            due_date: Set(self.due_date),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
    }
}

impl ToActiveModel<Entity> for UpdateAssignment {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let assignment = match AssignmentRepository::find_by_id(self.id).await {
            Ok(Some(assignment)) => assignment,
            Ok(None) => {
                return Err(DbErr::RecordNotFound(format!("Assignment ID {} not found", self.id)));
            }
            Err(err) => return Err(err),
        };

        validate_dates(
            self.available_from.unwrap_or(assignment.available_from),
            self.due_date.unwrap_or(assignment.due_date),
        )?;

        let mut active: ActiveModel = assignment.into();

        if let Some(name) = self.name {
            active.name = Set(name);
        }

        if let Some(description) = self.description {
            active.description = Set(Some(description));
        }

        if let Some(assignment_type) = self.assignment_type {
            active.assignment_type = Set(assignment_type.parse::<AssignmentType>().map_err(|_| DbErr::Custom("Invalid assignment_type".into()))?);
        }

        if let Some(available_from) = self.available_from {
            active.available_from = Set(available_from);
        }

        if let Some(due_date) = self.due_date {
            active.due_date = Set(due_date);
        }

        active.updated_at = Set(Utc::now());

        Ok(active)
    }
}

pub struct AssignmentService;

impl<'a> Service<'a, Entity, CreateAssignment, UpdateAssignment, AssignmentRepository> for AssignmentService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓

    // fn create(
    //         params: CreateAssignment,
    //     ) -> Pin<Box<dyn Future<Output = Result<<Entity as sea_orm::EntityTrait>::Model, AppError>> + Send + 'a>> {
    //     Box::pin(async move {
    //         AssignmentRepository::create(params.into_active_model().await?).await.map_err(AppError::from)
    //     })
    // }

    // fn update(
    //         params: UpdateAssignment,
    //     ) -> Pin<Box<dyn Future<Output = Result<<Entity as sea_orm::EntityTrait>::Model, AppError>> + Send + 'a>> {
    //     Box::pin(async move {
    //         AssignmentRepository::update(params.into_active_model().await?).await.map_err(AppError::from)
    //     })
    // }

    fn delete(
        id: i64,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>> {
         Box::pin(async move {
            let storage_root = env::var("ASSIGNMENT_STORAGE_ROOT")
                .unwrap_or_else(|_| "data/assignment_files".to_string());

            // TODO: Find a better way @reece
            let module_id = match AssignmentRepository::find_by_id(id).await? {
                Some(assignment) => assignment.module_id,
                None => return Err(DbErr::RecordNotFound(format!("Assignment ID {} not found", id)).into()),
            };

            let assignment_dir = PathBuf::from(storage_root)
                .join(format!("module_{module_id}"))
                .join(format!("assignment_{id}"));

            if assignment_dir.exists() {
                if let Err(e) = fs::remove_dir_all(&assignment_dir) {
                    eprintln!("Warning: Failed to delete assignment directory {:?}: {}", assignment_dir, e);
                }
            }

            AssignmentRepository::delete(id).await.map_err(AppError::from)
        })
    }
}

impl AssignmentService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

    pub async fn compute_readiness_report(
        module_id: i64,
        assignment_id: i64,
    ) -> Result<ReadinessReport, DbErr> {
        let config_present = AssignmentFileService::full_directory_path(
            module_id,
            assignment_id,
            &FileType::Config,
        )
        .read_dir()
        .map(|mut it| it.any(|f| f.is_ok()))
        .unwrap_or(false);

        let tasks_present = AssignmentTaskService::tasks_present(assignment_id).await;

        let main_present = AssignmentFileService::full_directory_path(
            module_id,
            assignment_id,
            &FileType::Main,
        )
        .read_dir()
        .map(|mut it| it.any(|f| f.is_ok()))
        .unwrap_or(false);

        let memo_present = AssignmentFileService::full_directory_path(
            module_id,
            assignment_id,
            &FileType::Memo,
        )
        .read_dir()
        .map(|mut it| it.any(|f| f.is_ok()))
        .unwrap_or(false);

        let makefile_present = AssignmentFileService::full_directory_path(
            module_id,
            assignment_id,
            &FileType::Makefile,
        )
        .read_dir()
        .map(|mut it| it.any(|f| f.is_ok()))
        .unwrap_or(false);

        let memo_output_present = {
            let base_path = AssignmentFileService::storage_root()
                .join(format!("module_{}", module_id))
                .join(format!("assignment_{}", assignment_id))
                .join("memo_output");

            if let Ok(entries) = fs::read_dir(&base_path) {
                entries.flatten().any(|entry| entry.path().is_file())
            } else {
                false
            }
        };

        let mark_allocator_present = AssignmentFileService::full_directory_path(
            module_id,
            assignment_id,
            &FileType::MarkAllocator,
        )
        .read_dir()
        .map(|it| {
            it.flatten()
                .any(|f| f.path().extension().map(|e| e == "json").unwrap_or(false))
        })
        .unwrap_or(false);

        Ok(ReadinessReport {
            config_present,
            tasks_present,
            main_present,
            memo_present,
            makefile_present,
            memo_output_present,
            mark_allocator_present,
        })
    }

    pub async fn try_transition_to_ready(
        module_id: i64,
        assignment_id: i64,
    ) -> Result<bool, DbErr> {
        let report = Self::compute_readiness_report(module_id, assignment_id).await?;

        if report.is_ready() {
            let mut active = AssignmentRepository::find_by_id(assignment_id).await?
                .ok_or(DbErr::RecordNotFound("Assignment not found".into()))?
                .into_active_model();

            if active.status.as_ref() == &Status::Setup {
                active.status = Set(Status::Ready);
                active.updated_at = Set(Utc::now());
                AssignmentRepository::update(active).await?;
            }
        }

        Ok(report.is_ready())
    }
}

fn validate_dates(available_from: DateTime<Utc>, due_date: DateTime<Utc>) -> Result<(), DbErr> {
    if due_date < available_from {
        Err(DbErr::Custom(
            "Due date cannot be before Available From date".into(),
        ))
    } else {
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use async_trait::async_trait;
//     use db::models::assignment::{Entity, Model};
//     use std::collections::HashMap;
//     use std::sync::Mutex;
//     use chrono::{TimeZone, Utc};

//     struct MockAssignmentRepository {
//         assignments: Mutex<HashMap<i64, Model>>,
//         next_id: Mutex<i64>,
//     }

//     impl MockAssignmentRepository {
//         fn new() -> Self {
//             Self {
//                 assignments: Mutex::new(HashMap::new()),
//                 next_id: Mutex::new(1),
//             }
//         }
//     }

//     #[async_trait]
//     impl Repository<Entity, AssignmentFilter> for MockAssignmentRepository {
//         async fn create(&self, active_model: assignment::ActiveModel) -> Result<Model, DbErr> {
//             let mut assignments = self.assignments.lock().unwrap();
//             let mut next_id = self.next_id.lock().unwrap();

//             let id = *next_id;
//             *next_id += 1;

//             let assignment = Model {
//                 id,
//                 module_id: active_model.module_id.unwrap(),
//                 name: active_model.name.unwrap(),
//                 description: active_model.description.unwrap(),
//                 assignment_type: active_model.assignment_type.unwrap(),
//                 status: active_model.status.unwrap(),
//                 available_from: active_model.available_from.unwrap(),
//                 due_date: active_model.due_date.unwrap(),
//                 config: None,
//                 created_at: chrono::Utc::now(),
//                 updated_at: chrono::Utc::now(),
//             };

//             assignments.insert(id, assignment.clone());
//             Ok(assignment)
//         }

//         async fn find_by_id(&self, id: i64) -> Result<Option<Model>, DbErr> {
//             let assignments = self.assignments.lock().unwrap();
//             Ok(assignments.get(&id).cloned())
//         }

//         async fn update(&self, active_model: assignment::ActiveModel) -> Result<Model, DbErr> {
//             let mut assignments = self.assignments.lock().unwrap();
//             let id = active_model.id.unwrap();

//             if let Some(assignment) = assignments.get_mut(&id) {
//                 assignment.name = active_model.name.unwrap();
//                 assignment.description = active_model.description.unwrap();
//                 assignment.assignment_type = active_model.assignment_type.unwrap();
//                 assignment.status = active_model.status.unwrap();
//                 assignment.available_from = active_model.available_from.unwrap();
//                 assignment.due_date = active_model.due_date.unwrap();
//                 assignment.updated_at = chrono::Utc::now();
//                 Ok(assignment.clone())
//             } else {
//                 Err(DbErr::RecordNotFound("Assignment not found".to_string()))
//             }
//         }

//         async fn delete(&self, id: i64) -> Result<(), DbErr> {
//             let mut assignments = self.assignments.lock().unwrap();
//             if assignments.remove(&id).is_some() {
//                 Ok(())
//             } else {
//                 Err(DbErr::RecordNotFound("Assignment not found".to_string()))
//             }
//         }

//         async fn filter(
//             &self,
//             filter_params: AssignmentFilter,
//             _page: u64,
//             _per_page: u64,
//             _sort_by: Option<String>,
//         ) -> Result<Vec<Model>, DbErr> {
//             let assignments = self.assignments.lock().unwrap();
//             match filter_params {
//                 AssignmentFilter::ModuleId(module_id) => {
//                     let filtered_assignments = assignments
//                         .values()
//                         .filter(|a| a.module_id == module_id)
//                         .cloned()
//                         .collect();
//                     Ok(filtered_assignments)
//                 }
//             }
//         }

//         async fn find_one(&self, filter_params: AssignmentFilter) -> Result<Option<Model>, DbErr> {
//             let assignments = self.assignments.lock().unwrap();
//             match filter_params {
//                 AssignmentFilter::ModuleId(module_id) => {
//                     let assignment = assignments
//                         .values()
//                         .find(|a| a.module_id == module_id)
//                         .cloned();
//                     Ok(assignment)
//                 }
//             }
//         }
//     }

//     fn sample_dates() -> (DateTime<Utc>, DateTime<Utc>) {
//         (
//             Utc.with_ymd_and_hms(2025, 6, 1, 9, 0, 0).unwrap(),
//             Utc.with_ymd_and_hms(2025, 6, 30, 17, 0, 0).unwrap(),
//         )
//     }

//     #[tokio::test]
//     async fn test_create_assignment() {
//         let repo = MockAssignmentRepository::new();
//         let service = AssignmentService::new(repo);
//         let (from, due) = sample_dates();

//         let assignment = service
//             .create_assignment(
//                 1,
//                 "Test Assignment",
//                 Some("Intro to Testing"),
//                 AssignmentType::Practical,
//                 from,
//                 due,
//             )
//             .await
//             .unwrap();

//         assert_eq!(assignment.module_id, 1);
//         assert_eq!(assignment.name, "Test Assignment");
//         assert_eq!(assignment.status, Status::Setup);
//     }

//     #[tokio::test]
//     async fn test_edit_assignment() {
//         let repo = MockAssignmentRepository::new();
//         let service = AssignmentService::new(repo);
//         let (from, due) = sample_dates();

//         let initial = service
//             .create_assignment(
//                 1,
//                 "Initial",
//                 Some("Initial Desc"),
//                 AssignmentType::Assignment,
//                 from,
//                 due,
//             )
//             .await
//             .unwrap();

//         let updated = service
//             .edit_assignment(
//                 initial.id,
//                 1,
//                 "Updated Name",
//                 Some("Updated Desc"),
//                 AssignmentType::Practical,
//                 from,
//                 due,
//             )
//             .await
//             .unwrap();

//         assert_eq!(updated.name, "Updated Name");
//         assert_eq!(updated.status, initial.status);
//     }
// }