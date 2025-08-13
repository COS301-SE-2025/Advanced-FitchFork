// use crate::service::Service;
// use crate::assignment_file_service::AssignmentFileService;
// use db::{
//     models::{assignment::{self, Model, /*Entity, AssignmentType,*/ Status, ReadinessReport}, assignment_file::FileType},
//     repositories::{repository::Repository, assignment_repository::AssignmentRepository},
//     filters::AssignmentFilter,
// };
// use sea_orm::{DbErr, Set, IntoActiveModel};
// use chrono::{DateTime, Utc};
// use std::{/*env,*/ fs, /*path::PathBuf*/};
// use serde::Deserialize;

// #[derive(Debug, Deserialize)]
// pub struct FilterReq {
//     pub page: Option<i32>,
//     pub per_page: Option<i32>,
//     pub sort: Option<String>,
//     pub query: Option<String>,
//     pub name: Option<String>,
//     pub assignment_type: Option<String>,
//     pub available_before: Option<String>,
//     pub available_after: Option<String>,
//     pub due_before: Option<String>,
//     pub due_after: Option<String>,
// }

// #[derive(Debug)]
// pub struct FilterResult {
//     pub assignments: Vec<Model>,
//     pub total: u64,
// }

// impl FilterResult {
//     pub fn new(assignments: Vec<Model>, total: u64) -> Self {
//         Self { assignments, total }
//     }
// }

// pub struct AssignmentService {
//     repo: AssignmentRepository,
// }

// impl<'a> Service<'a, assignment::Entity, AssignmentFilter, AssignmentRepository> for AssignmentService {
//     fn repository(&'a self) -> &'a AssignmentRepository {
//         &self.repo
//     }

//     // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓

//     fn create(
//         &self,
//         module_id: i64,
//         name: &str,
//         description: Option<&str>,
//         assignment_type: AssignmentType,
//         available_from: DateTime<Utc>,
//         due_date: DateTime<Utc>,
//     ) -> Result<Model, DbErr> {
//         Self::validate_dates(available_from, due_date)?;

//         let active_model = assignment::ActiveModel {
//             module_id: Set(module_id),
//             name: Set(name.to_string()),
//             description: Set(description.map(|d| d.to_string())),
//             assignment_type: Set(assignment_type),
//             status: Set(Status::Setup),
//             available_from: Set(available_from),
//             due_date: Set(due_date),
//             created_at: Set(Utc::now()),
//             updated_at: Set(Utc::now()),
//             ..Default::default()
//         };

//         self.repo.create(active_model).await
//     }

//     fn update(
//         &self,
//         id: i64,
//         name: &str,
//         description: Option<&str>,
//         assignment_type: AssignmentType,
//         available_from: DateTime<Utc>,
//         due_date: DateTime<Utc>,
//     ) -> Result<Model, DbErr> {
//         Self::validate_dates(available_from, due_date)?;

//         let assignment = self
//             .repo
//             .find_by_id(id)
//             .await?
//             .ok_or(DbErr::RecordNotFound("Assignment not found".to_string()))?;

//         let mut active: assignment::ActiveModel = assignment.into();
//         active.name = Set(name.to_string());
//         active.description = Set(description.map(|d| d.to_string()));
//         active.assignment_type = Set(assignment_type);
//         active.available_from = Set(available_from);
//         active.due_date = Set(due_date);
//         active.updated_at = Set(Utc::now());

//         self.repo.update(active).await
//     }

//     fn delete(
//         &self,
//         id: i64,
//         module_id: i64
//     ) -> Result<(), DbErr> {
//         self.repo.delete(id).await?;

//         let storage_root = env::var("ASSIGNMENT_STORAGE_ROOT")
//             .unwrap_or_else(|_| "data/assignment_files".to_string());

//         let assignment_dir = PathBuf::from(storage_root)
//             .join(format!("module_{}", module_id))
//             .join(format!("assignment_{}", id));

//         if assignment_dir.exists() {
//             if let Err(e) = fs::remove_dir_all(&assignment_dir) {
//                 eprintln!("Warning: Failed to delete assignment directory {:?}: {}", assignment_dir, e);
//             }
//         }

//         Ok(())
//     }

//     fn filter(
//         &self,
//         params: FilterReq,
//         module_id: i64,
//         page: u64,
//         per_page: u64,
//         sort_by: Option<String>,
//     ) -> Result<FilterResult, DbErr> {
//         use chrono::{DateTime, Utc};

//         let mut filter = AssignmentFilter::new().with_module_id(module_id);

//         if let Some(name) = params.name {
//             filter = filter.with_name(name);
//         }

//         if let Some(query) = params.query {
//             filter = filter.with_query(query);
//         }

//         if let Some(assignment_type_str) = params.assignment_type {
//             let assignment_type = assignment_type_str
//                 .parse::<AssignmentType>()
//                 .map_err(|_| DbErr::Custom("Invalid assignment_type".into()))?;
//             filter = filter.with_assignment_type(assignment_type);
//         }

//         let parse_datetime = |s: &str| -> Result<DateTime<Utc>, DbErr> {
//             DateTime::parse_from_rfc3339(s)
//                 .map(|dt| dt.with_timezone(&Utc))
//                 .map_err(|_| DbErr::Custom(format!("Invalid datetime format: {}", s)))
//         };

//         if let Some(before) = params.available_before {
//             let dt = parse_datetime(&before)?;
//             filter = filter.with_available_before(dt);
//         }

//         if let Some(after) = params.available_after {
//             let dt = parse_datetime(&after)?;
//             filter = filter.with_available_after(dt);
//         }

//         if let Some(before) = params.due_before {
//             let dt = parse_datetime(&before)?;
//             filter = filter.with_due_before(dt);
//         }

//         if let Some(after) = params.due_after {
//             let dt = parse_datetime(&after)?;
//             filter = filter.with_due_after(dt);
//         }

//         let total = <Self as Service<
//             '_,
//             Entity,
//             AssignmentFilter,
//             AssignmentRepository,
//         >>::count(self, filter.clone()).await?;

//         let assignments = <Self as Service<
//             '_,
//             Entity,
//             AssignmentFilter,
//             AssignmentRepository,
//         >>::filter(self, filter, page, per_page, sort_by).await?;

//         Ok(FilterResult::new(assignments, total))
//     }
// }

// impl AssignmentService {
//     pub fn new(repo: AssignmentRepository) -> Self {
//         Self { repo }
//     }

//     // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

//     fn validate_dates(available_from: DateTime<Utc>, due_date: DateTime<Utc>) -> Result<(), DbErr> {
//         if due_date < available_from {
//             Err(DbErr::Custom(
//                 "Due date cannot be before Available From date".into(),
//             ))
//         } else {
//             Ok(())
//         }
//     }

//     pub async fn compute_readiness_report<'a>(
//         &self,
//         module_id: i64,
//         assignment_id: i64,
//     ) -> Result<ReadinessReport, DbErr> {
//         let config_present = AssignmentFileService::full_directory_path(
//             module_id,
//             assignment_id,
//             &FileType::Config,
//         )
//         .read_dir()
//         .map(|mut it| it.any(|f| f.is_ok()))
//         .unwrap_or(false);

//         // TODO: Fix
//         // This will be passed as a parameter or fetched by a task-specific repository
//         let tasks_present = true; // Placeholder for now

//         let main_present = AssignmentFileService::full_directory_path(
//             module_id,
//             assignment_id,
//             &FileType::Main,
//         )
//         .read_dir()
//         .map(|mut it| it.any(|f| f.is_ok()))
//         .unwrap_or(false);

//         let memo_present = AssignmentFileService::full_directory_path(
//             module_id,
//             assignment_id,
//             &FileType::Memo,
//         )
//         .read_dir()
//         .map(|mut it| it.any(|f| f.is_ok()))
//         .unwrap_or(false);

//         let makefile_present = AssignmentFileService::full_directory_path(
//             module_id,
//             assignment_id,
//             &FileType::Makefile,
//         )
//         .read_dir()
//         .map(|mut it| it.any(|f| f.is_ok()))
//         .unwrap_or(false);

//         let memo_output_present = {
//             let base_path = AssignmentFileService::storage_root()
//                 .join(format!("module_{}", module_id))
//                 .join(format!("assignment_{}", assignment_id))
//                 .join("memo_output");

//             if let Ok(entries) = fs::read_dir(&base_path) {
//                 entries.flatten().any(|entry| entry.path().is_file())
//             } else {
//                 false
//             }
//         };

//         let mark_allocator_present = AssignmentFileService::full_directory_path(
//             module_id,
//             assignment_id,
//             &FileType::MarkAllocator,
//         )
//         .read_dir()
//         .map(|it| {
//             it.flatten()
//                 .any(|f| f.path().extension().map(|e| e == "json").unwrap_or(false))
//         })
//         .unwrap_or(false);

//         Ok(ReadinessReport {
//             config_present,
//             tasks_present,
//             main_present,
//             memo_present,
//             makefile_present,
//             memo_output_present,
//             mark_allocator_present,
//         })
//     }

//     pub async fn try_transition_to_ready<'a>(
//         &self,
//         module_id: i64,
//         assignment_id: i64,
//     ) -> Result<bool, DbErr> {
//         let report = self.compute_readiness_report(module_id, assignment_id).await?;

//         if report.is_ready() {
//             let mut active = self
//                 .repo
//                 .find_by_id(assignment_id)
//                 .await?
//                 .ok_or(DbErr::RecordNotFound("Assignment not found".into()))?
//                 .into_active_model();

//             if active.status.as_ref() == &Status::Setup {
//                 active.status = Set(Status::Ready);
//                 active.updated_at = Set(Utc::now());
//                 self.repo.update(active).await?;
//             }
//         }

//         Ok(report.is_ready())
//     }
// }

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