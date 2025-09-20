use crate::models::assignment;
use crate::models::assignment::Model as AssignmentModel;
use crate::models::user;
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait, QueryOrder};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use util::execution_config::ExecutionConfig;
use util::execution_config::GradingPolicy;
use util::paths::{ensure_parent_dir, storage_root};

/// Represents the status of a submission throughout its lifecycle
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "submission_status_enum"
)]
pub enum SubmissionStatus {
    /// Waiting for execution/marking
    #[sea_orm(string_value = "queued")]
    Queued,
    /// Code is compiling/building
    #[sea_orm(string_value = "running")]
    Running,
    /// Marking in progress
    #[sea_orm(string_value = "grading")]
    Grading,
    /// Marking complete (success)
    #[sea_orm(string_value = "graded")]
    Graded,
    /// File save error
    #[sea_orm(string_value = "failed_upload")]
    FailedUpload,
    /// Compilation failure
    #[sea_orm(string_value = "failed_compile")]
    FailedCompile,
    /// Runtime/test execution failure
    #[sea_orm(string_value = "failed_execution")]
    FailedExecution,
    /// Marking logic failure
    #[sea_orm(string_value = "failed_grading")]
    FailedGrading,
    /// Unexpected internal error
    #[sea_orm(string_value = "failed_internal")]
    FailedInternal,
}

impl Default for SubmissionStatus {
    fn default() -> Self {
        Self::Queued
    }
}

impl std::fmt::Display for SubmissionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_str = match self {
            SubmissionStatus::Queued => "queued",
            SubmissionStatus::Running => "running",
            SubmissionStatus::Grading => "grading",
            SubmissionStatus::Graded => "graded",
            SubmissionStatus::FailedUpload => "failed_upload",
            SubmissionStatus::FailedCompile => "failed_compile",
            SubmissionStatus::FailedExecution => "failed_execution",
            SubmissionStatus::FailedGrading => "failed_grading",
            SubmissionStatus::FailedInternal => "failed_internal",
        };
        write!(f, "{}", status_str)
    }
}

/// Represents a user's submission for a specific assignment.
///
/// Each submission is linked to one assignment and one user.
/// Timestamps are included to track when the submission was created and last updated.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "assignment_submissions")]
pub struct Model {
    /// Primary key of the submission.
    #[sea_orm(primary_key)]
    pub id: i64,
    /// ID of the related assignment.
    pub assignment_id: i64,
    /// ID of the user who submitted the assignment.
    pub user_id: i64,
    /// Attempt number
    pub attempt: i64,
    /// The score earned by the user.
    pub earned: i64,
    /// The total possible score.
    pub total: i64,
    /// The original filename uploaded by the user.
    pub filename: String,
    /// The hash of the submitted files.
    pub file_hash: String,
    /// Relative file path from the storage root.
    pub path: String,
    /// Is this submission a practice submission?
    pub is_practice: bool,
    /// Whether this submission should be ignored for grading/analytics.
    pub ignored: bool,
    /// Current status of the submission in the lifecycle.
    pub status: SubmissionStatus,
    /// Timestamp when the submission was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the submission was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Defines relationships between `assignment_submissions` and other tables.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Link to the related assignment.
    #[sea_orm(
        belongs_to = "super::assignment::Entity",
        from = "Column::AssignmentId",
        to = "super::assignment::Column::Id"
    )]
    Assignment,

    /// Link to the user who submitted the assignment.
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

/// Custom behavior for the active model (currently using default behavior).
impl ActiveModelBehavior for ActiveModel {}

impl Related<user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Model {
    /// Computes the absolute path to the stored file on disk.
    ///
    /// # Returns
    /// - `PathBuf` pointing to the file location.
    pub fn full_path(&self) -> std::path::PathBuf {
        storage_root().join(&self.path)
    }

    /// Saves a file to disk and creates or updates its metadata in the database.
    ///
    /// This method:
    /// 1. Creates a temporary DB entry.
    /// 2. Looks up the associated assignment and module.
    /// 3. Saves the file with a generated name on disk.
    /// 4. Updates the DB entry with the file path (relative to STORAGE_ROOT).
    pub async fn save_file(
        db: &DatabaseConnection,
        assignment_id: i64,
        user_id: i64,
        attempt: i64,
        earned: i64,
        total: i64,
        is_practice: bool,
        filename: &str,
        file_hash: &str,
        bytes: &[u8],
    ) -> Result<Self, DbErr> {
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
    }

    /// Loads the contents of the stored file from disk.
    ///
    /// # Returns
    /// - `Ok(Vec<u8>)`: The file contents as bytes.
    /// - `Err(std::io::Error)`: If reading the file fails.
    pub fn load_file(&self) -> Result<Vec<u8>, std::io::Error> {
        fs::read(self.full_path())
    }

    /// Deletes the file from disk without removing the database record.
    ///
    /// # Returns
    /// - `Ok(())`: If the file was successfully deleted.
    /// - `Err(std::io::Error)`: If the file deletion failed.
    pub fn delete_file_only(&self) -> Result<(), std::io::Error> {
        fs::remove_file(self.full_path())
    }

    /// Find all submission IDs for a given assignment
    pub async fn find_by_assignment(
        assignment_id: i64,
        db: &DatabaseConnection,
    ) -> Result<Vec<i64>, DbErr> {
        let submissions = Entity::find()
            .filter(Column::AssignmentId.eq(assignment_id as i32))
            .all(db)
            .await?;

        Ok(submissions.into_iter().map(|s| s.id).collect())
    }

    pub async fn get_latest_submissions_for_assignment(
        db: &DatabaseConnection,
        assignment_id: i64,
    ) -> Result<Vec<Self>, DbErr> {
        let all = Entity::find()
            .filter(Column::AssignmentId.eq(assignment_id))
            .order_by_asc(Column::UserId)
            .order_by_desc(Column::Attempt)
            .all(db)
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

    pub async fn get_best_for_user(
        db: &DatabaseConnection,
        assignment: &AssignmentModel,
        user_id: i64,
    ) -> Result<Option<Self>, DbErr> {
        // fetch all non-ignored, non-practice submissions for this user
        let mut subs = Entity::find()
            .filter(Column::AssignmentId.eq(assignment.id))
            .filter(Column::UserId.eq(user_id))
            .filter(Column::Ignored.eq(false))
            .filter(Column::IsPractice.eq(false))
            .all(db)
            .await?;

        if subs.is_empty() {
            return Ok(None);
        }

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
            | SubmissionStatus::FailedInternal => {
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

#[cfg(test)]
mod tests {
    use super::Model;
    use crate::models::{assignment::AssignmentType, user::Model as UserModel};
    use crate::test_utils::setup_test_db;
    use chrono::Utc;
    use sea_orm::{ActiveModelTrait, Set};
    use util::paths::storage_root;
    use util::test_helpers::setup_test_storage_root;

    fn fake_bytes() -> Vec<u8> {
        vec![0x50, 0x4B, 0x03, 0x04] // ZIP header (PK...)
    }

    #[tokio::test]
    async fn test_save_load_delete_submission_file() {
        let _tmp = setup_test_storage_root();
        let db = setup_test_db().await;

        // Create dummy user
        let user = UserModel::create(
            &db,
            "u63963920",
            "testuser12y4f@example.com",
            "password123",
            false,
        )
        .await
        .expect("Failed to insert user");

        // Create dummy module
        let module = crate::models::module::ActiveModel {
            code: Set("COS629".to_string()),
            year: Set(9463),
            description: Set(Some("aqw".to_string())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("Failed to insert module");

        // Create dummy assignment
        let assignment = crate::models::assignment::Model::create(
            &db,
            module.id,
            "Test fsh",
            Some("Description"),
            AssignmentType::Practical,
            Utc::now(),
            Utc::now(),
        )
        .await
        .expect("Failed to insert assignment");

        // Create dummy assignment_submission
        let _ = crate::models::assignment_submission::ActiveModel {
            assignment_id: Set(assignment.id),
            user_id: Set(user.id),
            attempt: Set(1),
            earned: Set(10),
            total: Set(10),
            filename: Set("solution.zip".to_string()),
            file_hash: Set("hash123#".to_string()),
            path: Set("".to_string()),
            is_practice: Set(false),
            ignored: Set(false),
            status: Set(super::SubmissionStatus::Queued),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("Failed to insert submission");

        // Save file via submission
        let content = fake_bytes();
        let file = Model::save_file(
            &db,
            assignment.id,
            user.id,
            6,
            10,
            10,
            false,
            "solution.zip",
            "hash123#",
            &content,
        )
        .await
        .expect("Failed to save file");

        assert_eq!(file.assignment_id, assignment.id);
        assert_eq!(file.user_id, user.id);
        assert_eq!(file.status, super::SubmissionStatus::Queued);
        assert!(file.path.contains("assignment_submissions"));

        // Confirm file written
        let full_path = storage_root().join(&file.path);
        assert!(full_path.exists());

        // Load content and verify
        let loaded = file.load_file().expect("Failed to load file");
        assert_eq!(loaded, content);

        // Delete file
        file.delete_file_only().expect("Failed to delete file");
        assert!(!full_path.exists());
    }

    #[tokio::test]
    async fn test_submission_status_defaults() {
        let _tmp = setup_test_storage_root();
        let db = setup_test_db().await;

        // Create dummy user
        let user = UserModel::create(
            &db,
            "u12345678",
            "testuser@example.com",
            "password123",
            false,
        )
        .await
        .expect("Failed to insert user");

        // Create dummy module
        let module = crate::models::module::ActiveModel {
            code: Set("COS101".to_string()),
            year: Set(2025),
            description: Set(Some("Test Module".to_string())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("Failed to insert module");

        // Create dummy assignment
        let assignment = crate::models::assignment::Model::create(
            &db,
            module.id,
            "Test Assignment",
            Some("Test Description"),
            AssignmentType::Practical,
            Utc::now(),
            Utc::now(),
        )
        .await
        .expect("Failed to insert assignment");

        // Test that new submissions default to Queued status
        let content = fake_bytes();
        let submission = Model::save_file(
            &db,
            assignment.id,
            user.id,
            1,
            0,
            10,
            false,
            "test.zip",
            "test_hash",
            &content,
        )
        .await
        .expect("Failed to save file");

        assert_eq!(submission.status, super::SubmissionStatus::Queued);
    }

    #[tokio::test]
    async fn test_submission_status_transitions() {
        let _tmp = setup_test_storage_root();
        let db = setup_test_db().await;

        // Create dummy user
        let user = UserModel::create(
            &db,
            "u87654321",
            "statustest@example.com",
            "password123",
            false,
        )
        .await
        .expect("Failed to insert user");

        // Create dummy module
        let module = crate::models::module::ActiveModel {
            code: Set("COS202".to_string()),
            year: Set(2025),
            description: Set(Some("Status Test Module".to_string())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("Failed to insert module");

        // Create dummy assignment
        let assignment = crate::models::assignment::Model::create(
            &db,
            module.id,
            "Status Test Assignment",
            Some("Status Test Description"),
            AssignmentType::Practical,
            Utc::now(),
            Utc::now(),
        )
        .await
        .expect("Failed to insert assignment");

        // Create initial submission
        let content = fake_bytes();
        let submission = Model::save_file(
            &db,
            assignment.id,
            user.id,
            1,
            0,
            10,
            false,
            "status_test.zip",
            "status_hash",
            &content,
        )
        .await
        .expect("Failed to save file");

        // Test status transition to Running
        let updated = Model::set_running(&db, submission.id)
            .await
            .expect("Failed to set running status");
        assert_eq!(updated.status, super::SubmissionStatus::Running);

        // Test status transition to Grading
        let updated = Model::set_grading(&db, submission.id)
            .await
            .expect("Failed to set grading status");
        assert_eq!(updated.status, super::SubmissionStatus::Grading);

        // Test status transition to Graded
        let updated = Model::set_graded(&db, submission.id)
            .await
            .expect("Failed to set graded status");
        assert_eq!(updated.status, super::SubmissionStatus::Graded);

        // Test status transition to various failure states
        let updated = Model::set_failed(&db, submission.id, super::SubmissionStatus::FailedCompile)
            .await
            .expect("Failed to set failed compile status");
        assert_eq!(updated.status, super::SubmissionStatus::FailedCompile);

        let updated =
            Model::set_failed(&db, submission.id, super::SubmissionStatus::FailedExecution)
                .await
                .expect("Failed to set failed execution status");
        assert_eq!(updated.status, super::SubmissionStatus::FailedExecution);

        // Test invalid failure type
        let result = Model::set_failed(&db, submission.id, super::SubmissionStatus::Queued).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_submission_status_helpers() {
        let _tmp = setup_test_storage_root();
        let db = setup_test_db().await;

        // Create dummy user
        let user = UserModel::create(
            &db,
            "u11223344",
            "helpertest@example.com",
            "password123",
            false,
        )
        .await
        .expect("Failed to insert user");

        // Create dummy module
        let module = crate::models::module::ActiveModel {
            code: Set("COS303".to_string()),
            year: Set(2025),
            description: Set(Some("Helper Test Module".to_string())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("Failed to insert module");

        // Create dummy assignment
        let assignment = crate::models::assignment::Model::create(
            &db,
            module.id,
            "Helper Test Assignment",
            Some("Helper Test Description"),
            AssignmentType::Practical,
            Utc::now(),
            Utc::now(),
        )
        .await
        .expect("Failed to insert assignment");

        // Test with queued submission
        let content = fake_bytes();
        let submission = Model::save_file(
            &db,
            assignment.id,
            user.id,
            1,
            0,
            10,
            false,
            "helper_test.zip",
            "helper_hash",
            &content,
        )
        .await
        .expect("Failed to save file");

        assert!(!submission.is_failed());
        assert!(!submission.is_complete());
        assert!(submission.is_in_progress());

        // Test with failed submission
        let failed_submission =
            Model::set_failed(&db, submission.id, super::SubmissionStatus::FailedGrading)
                .await
                .expect("Failed to set failed status");

        assert!(failed_submission.is_failed());
        assert!(failed_submission.is_complete());
        assert!(!failed_submission.is_in_progress());

        // Test with graded submission
        let graded_submission = Model::set_graded(&db, submission.id)
            .await
            .expect("Failed to set graded status");

        assert!(!graded_submission.is_failed());
        assert!(graded_submission.is_complete());
        assert!(!graded_submission.is_in_progress());
    }
}
