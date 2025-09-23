use crate::service::{AppError, Service, ToActiveModel};
use chrono::Utc;
use db::{
    models::assignment::{Column as AssignmentColumn, Entity as AssignmentEntity},
    models::assignment_submission::{
        ActiveModel, Column as AssignmentSubmissionColumn, Entity as AssignmentSubmissionEntity,
        Model,
    },
    repository::Repository,
};
use sea_orm::{DbErr, Set};
use std::collections::HashSet;
use std::path::PathBuf;
use std::{env, fs};
use util::execution_config::{GradingPolicy, ExecutionConfig};
use util::filters::FilterParam;

pub use db::models::assignment_submission::Model as AssignmentSubmission;
pub use db::models::assignment_submission::SubmissionStatus;

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
    pub earned: Option<i64>,
    pub total: Option<i64>,
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
        let submission =
            match Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::find_by_id(
                self.id,
            )
            .await
            {
                Ok(Some(submission)) => submission,
                Ok(None) => {
                    return Err(AppError::from(DbErr::RecordNotFound(format!(
                        "Submission not found for ID {}",
                        self.id
                    ))));
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

impl<'a>
    Service<
        'a,
        AssignmentSubmissionEntity,
        AssignmentSubmissionColumn,
        CreateAssignmentSubmission,
        UpdateAssignmentSubmission,
    > for AssignmentSubmissionService
{
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓

    fn create(
        params: CreateAssignmentSubmission,
    ) -> std::pin::Pin<
        Box<
            dyn std::prelude::rust_2024::Future<
                    Output = Result<
                        <AssignmentSubmissionEntity as sea_orm::EntityTrait>::Model,
                        AppError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            if earned > total {
                return Err(DbErr::Custom(
                    "Earned score cannot be greater than total score".into(),
                ));
            }

            let now = Utc::now();

            // Step 1: Insert placeholder model
            let partial = ActiveModel {
                assignment_id: Set(assignment_id),
                user_id: Set(user_id),
                attempt: Set(attempt),
                is_practice: Set(is_practice),
                ignored: Set(false),
                earned: Set(earned),
                total: Set(total),
                filename: Set(filename.to_string()),
                file_hash: Set(file_hash.to_string()),
                path: Set(String::new()),
                status: Set(SubmissionStatus::Queued),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            };

            let inserted: Model = partial.insert(db).await?;

            // Step 2: Lookup module_id
            let assignment = assignment::Entity::find_by_id(assignment_id)
                .one(db)
                .await?
                .ok_or_else(|| DbErr::Custom("Assignment not found".into()))?;
            let module_id = assignment.module_id;

            // Step 3: Derive extension from the *uploaded filename* (no content sniffing)
            let ext = PathBuf::from(filename)
                .extension()
                .map(|e| e.to_string_lossy().to_string());

            // Step 4: Build target path via utilities and write file
            let file_path = util::paths::submission_file_path(
                module_id,
                assignment_id,
                user_id,
                attempt,
                inserted.id,
                ext.as_deref(),
            );
            ensure_parent_dir(&file_path)
                .map_err(|e| DbErr::Custom(format!("Failed to create directory: {e}")))?;

            fs::write(&file_path, bytes)
                .map_err(|e| DbErr::Custom(format!("Failed to write file: {e}")))?;

            // Compute relative path from STORAGE_ROOT and persist
            let relative_path = file_path
                .strip_prefix(storage_root())
                .unwrap()
                .to_string_lossy()
                .to_string();

            // Step 5: Update DB with path
            let mut model: ActiveModel = inserted.into();
            model.path = Set(relative_path);
            model.updated_at = Set(Utc::now());

            model.update(db).await
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

    pub async fn full_path(id: i64) -> Result<PathBuf, DbErr> {
        let submission =
            Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::find_by_id(id)
                .await?
                .ok_or_else(|| DbErr::RecordNotFound(format!("Submission ID {} not found", id)))?;
        Ok(Self::storage_root().join(submission.path))
    }

    pub async fn load_file(id: i64) -> Result<Vec<u8>, std::io::Error> {
        let path = Self::full_path(id)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        fs::read(path)
    }

    pub async fn delete_file_only(id: i64) -> Result<(), std::io::Error> {
        let path = Self::full_path(id)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        fs::remove_file(path)
    }

    pub async fn find_by_assignment(assignment_id: i64) -> Result<Vec<i64>, DbErr> {
        let submissions =
            Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::find_all(
                &vec![FilterParam::eq("assignment_id", assignment_id)],
                &vec![],
                None,
            )
            .await?;

        Ok(submissions.into_iter().map(|s| s.id as i64).collect())
    }

    pub async fn get_latest_submissions_for_assignment(
        assignment_id: i64,
    ) -> Result<Vec<Model>, DbErr> {
        let all = Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::find_all(
            &vec![FilterParam::eq("assignment_id", assignment_id)],
            &vec![],
            Some("user_id,-attempt".to_string()),
        )
        .await?;

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
        let mut subs =
            Repository::<AssignmentSubmissionEntity, AssignmentSubmissionColumn>::find_all(
                &vec![
                    FilterParam::eq("assignment_id", assignment_id),
                    FilterParam::eq("user_id", user_id),
                    FilterParam::eq("ignored", false),
                    FilterParam::eq("is_practice", false),
                ],
                &vec![],
                None,
            )
            .await?;

        if subs.is_empty() {
            return Ok(None);
        }

        let assignment =
            Repository::<AssignmentEntity, AssignmentColumn>::find_by_id(assignment_id)
                .await?
                .ok_or_else(|| {
                    DbErr::RecordNotFound(format!("Assignment ID {} not found", assignment_id))
                })?;
        let cfg = assignment
            .config()
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

    /// Set the `ignored` flag for a submission by id and return the updated model.
    pub async fn set_ignored(
        db: &DatabaseConnection,
        submission_id: i64,
        ignored: bool,
    ) -> Result<Self, DbErr> {
        // Load existing
        let existing = Entity::find_by_id(submission_id)
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom(format!("Submission {submission_id} not found")))?;

        let mut am: ActiveModel = existing.into();
        am.ignored = Set(ignored);
        am.updated_at = Set(Utc::now());
        am.update(db).await
    }

    pub async fn get_selected_submissions_for_assignment(
        db: &DatabaseConnection,
        assignment: &AssignmentModel,
    ) -> Result<Vec<Self>, DbErr> {
        let all_for_assignment = Entity::find()
            .filter(Column::AssignmentId.eq(assignment.id))
            .all(db)
            .await?;

        let mut user_ids = HashSet::<i64>::new();
        for s in &all_for_assignment {
            user_ids.insert(s.user_id);
        }

        let mut chosen = Vec::with_capacity(user_ids.len());
        for uid in user_ids {
            if let Ok(Some(s)) = Model::get_best_for_user(db, assignment, uid).await {
                chosen.push(s);
            }
        }
        Ok(chosen)
    }

    /// Update the status of a submission and persist to database
    pub async fn update_status(
        db: &DatabaseConnection,
        submission_id: i64,
        new_status: SubmissionStatus,
    ) -> Result<Self, DbErr> {
        // Load existing submission
        let existing = Entity::find_by_id(submission_id)
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom(format!("Submission {submission_id} not found")))?;

        let mut active_model: ActiveModel = existing.into();
        active_model.status = Set(new_status);
        active_model.updated_at = Set(Utc::now());

        active_model.update(db).await
    }

    /// Set the status to running
    pub async fn set_running(db: &DatabaseConnection, submission_id: i64) -> Result<Self, DbErr> {
        Self::update_status(db, submission_id, SubmissionStatus::Running).await
    }

    /// Set the status to grading
    pub async fn set_grading(db: &DatabaseConnection, submission_id: i64) -> Result<Self, DbErr> {
        Self::update_status(db, submission_id, SubmissionStatus::Grading).await
    }

    /// Set the status to graded (successful completion)
    pub async fn set_graded(db: &DatabaseConnection, submission_id: i64) -> Result<Self, DbErr> {
        Self::update_status(db, submission_id, SubmissionStatus::Graded).await
    }

    /// Set the status to failed with the appropriate failure type
    pub async fn set_failed(
        db: &DatabaseConnection,
        submission_id: i64,
        failure_type: SubmissionStatus,
    ) -> Result<Self, DbErr> {
        // Validate that the provided status is a failure status
        match failure_type {
            SubmissionStatus::FailedUpload
            | SubmissionStatus::FailedCompile
            | SubmissionStatus::FailedExecution
            | SubmissionStatus::FailedGrading
            | SubmissionStatus::FailedInternal
            | SubmissionStatus::FailedDisallowedCode => {
                Self::update_status(db, submission_id, failure_type).await
            }
            _ => Err(DbErr::Custom(format!(
                "Invalid failure type: {:?}. Use a failed_* status.",
                failure_type
            ))),
        }
    }

    /// Check if the current status represents a failure state
    pub fn is_failed(&self) -> bool {
        matches!(
            self.status,
            SubmissionStatus::FailedUpload
                | SubmissionStatus::FailedCompile
                | SubmissionStatus::FailedExecution
                | SubmissionStatus::FailedGrading
                | SubmissionStatus::FailedInternal
                | SubmissionStatus::FailedDisallowedCode
        )
    }

    /// Check if the submission is complete (either graded or failed)
    pub fn is_complete(&self) -> bool {
        matches!(self.status, SubmissionStatus::Graded) || self.is_failed()
    }

    /// Check if the submission is currently in progress
    pub fn is_in_progress(&self) -> bool {
        matches!(
            self.status,
            SubmissionStatus::Queued | SubmissionStatus::Running | SubmissionStatus::Grading
        )
    }
}

// #[cfg(test)]
// mod tests {
//     use super::Model;
//     use crate::models::{assignment::AssignmentType, user::Model as UserModel};
//     use crate::test_utils::setup_test_db;
//     use chrono::Utc;
//     use sea_orm::{ActiveModelTrait, Set};
//     use util::paths::storage_root;
//     use util::test_helpers::setup_test_storage_root;

//     fn fake_bytes() -> Vec<u8> {
//         vec![0x50, 0x4B, 0x03, 0x04] // ZIP header (PK...)
//     }

//     #[tokio::test]
//     async fn test_save_load_delete_submission_file() {
//         let _tmp = setup_test_storage_root();
//         let db = setup_test_db().await;

//         // Create dummy user
//         let user = UserModel::create(
//             &db,
//             "u63963920",
//             "testuser12y4f@example.com",
//             "password123",
//             false,
//         )
//         .await
//         .expect("Failed to insert user");

//         // Create dummy module
//         let module = crate::models::module::ActiveModel {
//             code: Set("COS629".to_string()),
//             year: Set(9463),
//             description: Set(Some("aqw".to_string())),
//             created_at: Set(Utc::now()),
//             updated_at: Set(Utc::now()),
//             ..Default::default()
//         }
//         .insert(&db)
//         .await
//         .expect("Failed to insert module");

//         // Create dummy assignment
//         let assignment = crate::models::assignment::Model::create(
//             &db,
//             module.id,
//             "Test fsh",
//             Some("Description"),
//             AssignmentType::Practical,
//             Utc::now(),
//             Utc::now(),
//         )
//         .await
//         .expect("Failed to insert assignment");

//         // Create dummy assignment_submission
//         let _ = crate::models::assignment_submission::ActiveModel {
//             assignment_id: Set(assignment.id),
//             user_id: Set(user.id),
//             attempt: Set(1),
//             earned: Set(10),
//             total: Set(10),
//             filename: Set("solution.zip".to_string()),
//             file_hash: Set("hash123#".to_string()),
//             path: Set("".to_string()),
//             is_practice: Set(false),
//             ignored: Set(false),
//             status: Set(super::SubmissionStatus::Queued),
//             created_at: Set(Utc::now()),
//             updated_at: Set(Utc::now()),
//             ..Default::default()
//         }
//         .insert(&db)
//         .await
//         .expect("Failed to insert submission");

//         // Save file via submission
//         let content = fake_bytes();
//         let file = Model::save_file(
//             &db,
//             assignment.id,
//             user.id,
//             6,
//             10,
//             10,
//             false,
//             "solution.zip",
//             "hash123#",
//             &content,
//         )
//         .await
//         .expect("Failed to save file");

//         assert_eq!(file.assignment_id, assignment.id);
//         assert_eq!(file.user_id, user.id);
//         assert_eq!(file.status, super::SubmissionStatus::Queued);
//         assert!(file.path.contains("assignment_submissions"));

//         // Confirm file written
//         let full_path = storage_root().join(&file.path);
//         assert!(full_path.exists());

//         // Load content and verify
//         let loaded = file.load_file().expect("Failed to load file");
//         assert_eq!(loaded, content);

//         // Delete file
//         file.delete_file_only().expect("Failed to delete file");
//         assert!(!full_path.exists());
//     }

//     #[tokio::test]
//     async fn test_submission_status_defaults() {
//         let _tmp = setup_test_storage_root();
//         let db = setup_test_db().await;

//         // Create dummy user
//         let user = UserModel::create(
//             &db,
//             "u12345678",
//             "testuser@example.com",
//             "password123",
//             false,
//         )
//         .await
//         .expect("Failed to insert user");

//         // Create dummy module
//         let module = crate::models::module::ActiveModel {
//             code: Set("COS101".to_string()),
//             year: Set(2025),
//             description: Set(Some("Test Module".to_string())),
//             created_at: Set(Utc::now()),
//             updated_at: Set(Utc::now()),
//             ..Default::default()
//         }
//         .insert(&db)
//         .await
//         .expect("Failed to insert module");

//         // Create dummy assignment
//         let assignment = crate::models::assignment::Model::create(
//             &db,
//             module.id,
//             "Test Assignment",
//             Some("Test Description"),
//             AssignmentType::Practical,
//             Utc::now(),
//             Utc::now(),
//         )
//         .await
//         .expect("Failed to insert assignment");

//         // Test that new submissions default to Queued status
//         let content = fake_bytes();
//         let submission = Model::save_file(
//             &db,
//             assignment.id,
//             user.id,
//             1,
//             0,
//             10,
//             false,
//             "test.zip",
//             "test_hash",
//             &content,
//         )
//         .await
//         .expect("Failed to save file");

//         assert_eq!(submission.status, super::SubmissionStatus::Queued);
//     }

//     #[tokio::test]
//     async fn test_submission_status_transitions() {
//         let _tmp = setup_test_storage_root();
//         let db = setup_test_db().await;

//         // Create dummy user
//         let user = UserModel::create(
//             &db,
//             "u87654321",
//             "statustest@example.com",
//             "password123",
//             false,
//         )
//         .await
//         .expect("Failed to insert user");

//         // Create dummy module
//         let module = crate::models::module::ActiveModel {
//             code: Set("COS202".to_string()),
//             year: Set(2025),
//             description: Set(Some("Status Test Module".to_string())),
//             created_at: Set(Utc::now()),
//             updated_at: Set(Utc::now()),
//             ..Default::default()
//         }
//         .insert(&db)
//         .await
//         .expect("Failed to insert module");

//         // Create dummy assignment
//         let assignment = crate::models::assignment::Model::create(
//             &db,
//             module.id,
//             "Status Test Assignment",
//             Some("Status Test Description"),
//             AssignmentType::Practical,
//             Utc::now(),
//             Utc::now(),
//         )
//         .await
//         .expect("Failed to insert assignment");

//         // Create initial submission
//         let content = fake_bytes();
//         let submission = Model::save_file(
//             &db,
//             assignment.id,
//             user.id,
//             1,
//             0,
//             10,
//             false,
//             "status_test.zip",
//             "status_hash",
//             &content,
//         )
//         .await
//         .expect("Failed to save file");

//         // Test status transition to Running
//         let updated = Model::set_running(&db, submission.id)
//             .await
//             .expect("Failed to set running status");
//         assert_eq!(updated.status, super::SubmissionStatus::Running);

//         // Test status transition to Grading
//         let updated = Model::set_grading(&db, submission.id)
//             .await
//             .expect("Failed to set grading status");
//         assert_eq!(updated.status, super::SubmissionStatus::Grading);

//         // Test status transition to Graded
//         let updated = Model::set_graded(&db, submission.id)
//             .await
//             .expect("Failed to set graded status");
//         assert_eq!(updated.status, super::SubmissionStatus::Graded);

//         // Test status transition to various failure states
//         let updated = Model::set_failed(&db, submission.id, super::SubmissionStatus::FailedCompile)
//             .await
//             .expect("Failed to set failed compile status");
//         assert_eq!(updated.status, super::SubmissionStatus::FailedCompile);

//         let updated =
//             Model::set_failed(&db, submission.id, super::SubmissionStatus::FailedExecution)
//                 .await
//                 .expect("Failed to set failed execution status");
//         assert_eq!(updated.status, super::SubmissionStatus::FailedExecution);

//         // Test invalid failure type
//         let result = Model::set_failed(&db, submission.id, super::SubmissionStatus::Queued).await;
//         assert!(result.is_err());
//     }

//     #[tokio::test]
//     async fn test_submission_status_helpers() {
//         let _tmp = setup_test_storage_root();
//         let db = setup_test_db().await;

//         // Create dummy user
//         let user = UserModel::create(
//             &db,
//             "u11223344",
//             "helpertest@example.com",
//             "password123",
//             false,
//         )
//         .await
//         .expect("Failed to insert user");

//         // Create dummy module
//         let module = crate::models::module::ActiveModel {
//             code: Set("COS303".to_string()),
//             year: Set(2025),
//             description: Set(Some("Helper Test Module".to_string())),
//             created_at: Set(Utc::now()),
//             updated_at: Set(Utc::now()),
//             ..Default::default()
//         }
//         .insert(&db)
//         .await
//         .expect("Failed to insert module");

//         // Create dummy assignment
//         let assignment = crate::models::assignment::Model::create(
//             &db,
//             module.id,
//             "Helper Test Assignment",
//             Some("Helper Test Description"),
//             AssignmentType::Practical,
//             Utc::now(),
//             Utc::now(),
//         )
//         .await
//         .expect("Failed to insert assignment");

//         // Test with queued submission
//         let content = fake_bytes();
//         let submission = Model::save_file(
//             &db,
//             assignment.id,
//             user.id,
//             1,
//             0,
//             10,
//             false,
//             "helper_test.zip",
//             "helper_hash",
//             &content,
//         )
//         .await
//         .expect("Failed to save file");

//         assert!(!submission.is_failed());
//         assert!(!submission.is_complete());
//         assert!(submission.is_in_progress());

//         // Test with failed submission
//         let failed_submission =
//             Model::set_failed(&db, submission.id, super::SubmissionStatus::FailedGrading)
//                 .await
//                 .expect("Failed to set failed status");

//         assert!(failed_submission.is_failed());
//         assert!(failed_submission.is_complete());
//         assert!(!failed_submission.is_in_progress());

//         // Test with graded submission
//         let graded_submission = Model::set_graded(&db, submission.id)
//             .await
//             .expect("Failed to set graded status");

//         assert!(!graded_submission.is_failed());
//         assert!(graded_submission.is_complete());
//         assert!(!graded_submission.is_in_progress());
//     }
// }
