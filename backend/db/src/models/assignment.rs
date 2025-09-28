//! Entity and business logic for managing assignments.
//!
//! This module defines the `Assignment` model, its relations, and
//! methods for creating, editing, and filtering assignments.

use crate::models::assignment_file::{FileType, Model as AssignmentFileModel};
use crate::models::assignment_submission::{Column as SubmissionCol, Entity as SubmissionEntity};
use crate::models::assignment_task::{Column as TaskColumn, Entity as TaskEntity};
use crate::models::moss_report;
use crate::models::user_module_role::{
    Column as UserModuleRoleCol, Entity as UserModuleRoleEntity, Role as ModuleRole,
};
use chrono::{DateTime, Utc};
use ipnet::IpNet;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DbErr, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::net::IpAddr;
use strum::{Display, EnumIter, EnumString};
use util::execution_config::ExecutionConfig;
use util::execution_config::SubmissionMode;
use util::paths::{assignment_dir, interpreter_dir, memo_output_dir};

/// Assignment model representing the `assignments` table in the database.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "assignments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub module_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub assignment_type: AssignmentType,
    pub status: Status,
    pub available_from: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Defines the relationship between `Assignment` and `Module`.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::module::Entity",
        from = "Column::ModuleId",
        to = "super::module::Column::Id"
    )]
    Module,

    #[sea_orm(has_many = "super::moss_report::Entity")]
    MossReports,
}

impl Related<moss_report::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MossReports.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(
    Debug, Clone, PartialEq, Display, EnumIter, EnumString, Serialize, Deserialize, DeriveActiveEnum,
)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "assignment_type_enum"
)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum AssignmentType {
    #[sea_orm(string_value = "assignment")]
    Assignment,

    #[sea_orm(string_value = "practical")]
    Practical,
}

#[derive(
    Debug, Clone, PartialEq, Display, EnumIter, EnumString, Serialize, Deserialize, DeriveActiveEnum,
)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "assignment_status_enum"
)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum Status {
    #[sea_orm(string_value = "setup")]
    Setup,
    #[sea_orm(string_value = "ready")]
    Ready,
    #[sea_orm(string_value = "open")]
    Open,
    #[sea_orm(string_value = "closed")]
    Closed,
    #[sea_orm(string_value = "archived")]
    Archived,
}

/// Detailed report of assignment readiness state.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReadinessReport {
    pub submission_mode: SubmissionMode,
    pub config_present: bool,
    pub tasks_present: bool,
    pub main_present: bool,
    pub interpreter_present: bool,
    pub memo_present: bool,
    pub makefile_present: bool,
    pub memo_output_present: bool,
    pub mark_allocator_present: bool,
}

impl ReadinessReport {
    /// Readiness is conditional:
    /// - Manual         -> require main + memo_output + mark_allocator
    /// - GATLAM/RNG/CC  -> require interpreter
    pub fn is_ready(&self) -> bool {
        // common across all modes
        let base_ok =
            self.config_present && self.tasks_present && self.memo_present && self.makefile_present;

        if !base_ok {
            return false;
        }

        match self.submission_mode {
            util::execution_config::SubmissionMode::Manual => {
                self.main_present && self.memo_output_present && self.mark_allocator_present
            }
            util::execution_config::SubmissionMode::GATLAM
            | util::execution_config::SubmissionMode::RNG
            | util::execution_config::SubmissionMode::CodeCoverage => self.interpreter_present,
        }
    }
}

/// Quick summary object for attempts.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AttemptsSummary {
    pub used: u32,
    pub max: u32,
    pub remaining: u32,
    pub limit_attempts: bool,
}

impl Model {
    pub async fn create(
        db: &DatabaseConnection,
        module_id: i64,
        name: &str,
        description: Option<&str>,
        assignment_type: AssignmentType,
        available_from: DateTime<Utc>,
        due_date: DateTime<Utc>,
    ) -> Result<Self, DbErr> {
        Self::validate_dates(available_from, due_date)?;

        let active = ActiveModel {
            module_id: Set(module_id),
            name: Set(name.to_string()),
            description: Set(description.map(|d| d.to_string())),
            assignment_type: Set(assignment_type),
            status: Set(Status::Setup),
            available_from: Set(available_from),
            due_date: Set(due_date),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        let created = active.insert(db).await?;

        // auto-create default config.json (mirror on disk + DB record)
        let default_config = ExecutionConfig::default_config();
        match serde_json::to_vec(&default_config) {
            Ok(bytes) => {
                if let Err(e) = AssignmentFileModel::save_file(
                    db,
                    created.id,
                    module_id,
                    FileType::Config,
                    "config.json",
                    &bytes,
                )
                .await
                {
                    eprintln!(
                        "Warning: failed to store default execution config record for assignment {}: {}",
                        created.id, e
                    );

                    if let Err(e) = default_config.save(module_id, created.id) {
                        eprintln!(
                            "Warning: failed to save default execution config for assignment {}: {}",
                            created.id, e
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: failed to serialize default execution config for assignment {}: {}",
                    created.id, e
                );

                if let Err(e) = default_config.save(module_id, created.id) {
                    eprintln!(
                        "Warning: failed to save default execution config for assignment {}: {}",
                        created.id, e
                    );
                }
            }
        }

        Ok(created)
    }

    pub async fn edit(
        db: &DatabaseConnection,
        id: i64,
        module_id: i64,
        name: &str,
        description: Option<&str>,
        assignment_type: AssignmentType,
        available_from: DateTime<Utc>,
        due_date: DateTime<Utc>,
    ) -> Result<Self, DbErr> {
        Self::validate_dates(available_from, due_date)?;

        let mut assignment = Entity::find()
            .filter(Column::Id.eq(id))
            .filter(Column::ModuleId.eq(module_id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Assignment not found".to_string()))?
            .into_active_model();

        assignment.name = Set(name.to_string());
        assignment.description = Set(description.map(|d| d.to_string()));
        assignment.assignment_type = Set(assignment_type);
        assignment.available_from = Set(available_from);
        assignment.due_date = Set(due_date);
        assignment.updated_at = Set(Utc::now());

        assignment.update(db).await
    }

    pub async fn delete(db: &DatabaseConnection, id: i32, module_id: i32) -> Result<(), DbErr> {
        let Some(model) = Entity::find()
            .filter(Column::Id.eq(id))
            .filter(Column::ModuleId.eq(module_id))
            .one(db)
            .await?
        else {
            return Err(DbErr::RecordNotFound(format!(
                "Assignment {id} in module {module_id} not found"
            )));
        };

        let active = model.into_active_model();
        active.delete(db).await?;

        let dir = assignment_dir(module_id as i64, id as i64);
        if dir.exists() {
            if let Err(e) = fs::remove_dir_all(&dir) {
                eprintln!(
                    "Warning: Failed to delete assignment directory {:?}: {}",
                    dir, e
                );
            }
        }

        Ok(())
    }

    pub async fn filter(
        db: &DatabaseConnection,
        page: u64,
        per_page: u64,
        sort_by: Option<String>,
        query: Option<String>,
    ) -> Result<Vec<Self>, DbErr> {
        let mut query_builder = Entity::find();

        if let Some(q) = query {
            let pattern = format!("%{}%", q.to_lowercase());
            query_builder = query_builder.filter(
                Condition::any()
                    .add(Expr::cust("LOWER(name)").like(&pattern))
                    .add(Expr::cust("LOWER(description)").like(&pattern)),
            );
        }

        if let Some(sort) = sort_by {
            let (column, asc) = if sort.starts_with('-') {
                (&sort[1..], false)
            } else {
                (sort.as_str(), true)
            };

            query_builder = match column {
                "name" => {
                    if asc {
                        query_builder.order_by_asc(Column::Name)
                    } else {
                        query_builder.order_by_desc(Column::Name)
                    }
                }
                "due_date" => {
                    if asc {
                        query_builder.order_by_asc(Column::DueDate)
                    } else {
                        query_builder.order_by_desc(Column::DueDate)
                    }
                }
                "available_from" => {
                    if asc {
                        query_builder.order_by_asc(Column::AvailableFrom)
                    } else {
                        query_builder.order_by_desc(Column::AvailableFrom)
                    }
                }
                _ => query_builder,
            };
        }

        query_builder
            .paginate(db, per_page)
            .fetch_page(page - 1)
            .await
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

    /// Computes a detailed readiness report for an assignment, with requirements
    /// that adapt to the configured `SubmissionMode`.
    ///
    /// This function verifies the presence of all expected resources:
    /// - At least one **task** exists in the database.
    /// - A **configuration** file (`config.json` under `.../config/`) exists.
    /// - A **memo** file exists.
    /// - A **makefile** exists.
    /// - At least one **memo output** file exists.
    /// - A JSON **mark allocator** file exists.
    /// - **Main** or **Interpreter** presence is required **conditionally**:
    ///     - If `SubmissionMode::Manual`  → **main** file must be present.
    ///     - If `SubmissionMode::GATLAM`  → **interpreter** must be present.
    ///     - Other modes (e.g., `RNG`, `CodeCoverage`) do **not** require main/interpreter.
    ///
    /// The returned [`ReadinessReport`] includes:
    /// - `submission_mode`: the mode resolved from `config.json` (or default if missing/invalid).
    /// - Boolean flags for each component, including `main_present` and `interpreter_present`.
    /// - `is_ready()` that applies the conditional rule above.
    ///
    /// This function only checks readiness — it does **not** modify the assignment's status.
    ///
    /// # Arguments
    /// * `db` — Database connection.
    /// * `module_id` — The module ID.
    /// * `assignment_id` — The assignment ID.
    ///
    /// # Returns
    /// * `Ok(ReadinessReport)` with per-component readiness details.
    /// * `Err(DbErr)` if a database error occurs while checking tasks.
    ///
    /// # Notes
    /// File presence is checked on the filesystem under canonical directories.
    /// Missing directories or I/O errors are treated as absence of the respective component.
    pub async fn compute_readiness_report(
        db: &DatabaseConnection,
        module_id: i64,
        assignment_id: i64,
    ) -> Result<ReadinessReport, DbErr> {
        // presence checks (unchanged ones kept)
        let config_present =
            AssignmentFileModel::full_directory_path(module_id, assignment_id, &FileType::Config)
                .read_dir()
                .map(|mut it| it.any(|f| f.is_ok()))
                .unwrap_or(false);

        let tasks_present = TaskEntity::find()
            .filter(TaskColumn::AssignmentId.eq(assignment_id))
            .limit(1)
            .all(db)
            .await
            .map(|tasks| !tasks.is_empty())
            .unwrap_or(false);

        let main_present =
            AssignmentFileModel::full_directory_path(module_id, assignment_id, &FileType::Main)
                .read_dir()
                .map(|mut it| it.any(|f| f.is_ok()))
                .unwrap_or(false);

        // interpreter presence
        let interpreter_present = interpreter_dir(module_id, assignment_id)
            .read_dir()
            .map(|mut it| it.any(|f| f.is_ok()))
            .unwrap_or(false);

        let memo_present =
            AssignmentFileModel::full_directory_path(module_id, assignment_id, &FileType::Memo)
                .read_dir()
                .map(|mut it| it.any(|f| f.is_ok()))
                .unwrap_or(false);

        let makefile_present =
            AssignmentFileModel::full_directory_path(module_id, assignment_id, &FileType::Makefile)
                .read_dir()
                .map(|mut it| it.any(|f| f.is_ok()))
                .unwrap_or(false);

        let memo_output_present = {
            let base_path = memo_output_dir(module_id, assignment_id);
            if let Ok(entries) = fs::read_dir(&base_path) {
                entries.flatten().any(|entry| entry.path().is_file())
            } else {
                false
            }
        };

        let mark_allocator_present = AssignmentFileModel::full_directory_path(
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

        // Determine submission mode: prefer on-disk config.json; fallback to default
        let submission_mode = ExecutionConfig::get_execution_config(module_id, assignment_id)
            .map(|c| c.project.submission_mode)
            .unwrap_or_else(|_| ExecutionConfig::default_config().project.submission_mode);

        Ok(ReadinessReport {
            submission_mode,
            config_present,
            tasks_present,
            main_present,
            interpreter_present,
            memo_present,
            makefile_present,
            memo_output_present,
            mark_allocator_present,
        })
    }

    /// Attempts to transition an assignment to `Ready` state if all readiness conditions are met.
    ///
    /// This function:
    /// - Computes a full readiness report for the assignment.
    /// - If all components are present (`is_ready` == true), it checks the current status.
    /// - If the current status is `Setup`, it updates the status to `Ready` and updates `updated_at`.
    /// - If already in `Ready`, `Open`, etc., it does not change the status.
    ///
    /// # Arguments
    /// * `db` - A reference to the database connection.
    /// * `module_id` - The ID of the module to which the assignment belongs.
    /// * `assignment_id` - The ID of the assignment.
    ///
    /// # Returns
    /// * `Ok(true)` if the assignment is fully ready (regardless of whether the status was updated).
    /// * `Ok(false)` if the assignment is not ready.
    /// * `Err(DbErr)` if a database error occurs.
    pub async fn try_transition_to_ready(
        db: &DatabaseConnection,
        module_id: i64,
        assignment_id: i64,
    ) -> Result<bool, DbErr> {
        let report = Self::compute_readiness_report(db, module_id, assignment_id).await?;

        if report.is_ready() {
            let mut active = Entity::find()
                .filter(Column::Id.eq(assignment_id))
                .filter(Column::ModuleId.eq(module_id))
                .one(db)
                .await?
                .ok_or(DbErr::RecordNotFound("Assignment not found".into()))?
                .into_active_model();

            if active.status.as_ref() == &Status::Setup {
                active.status = Set(Status::Ready);
                active.updated_at = Set(Utc::now());
                active.update(db).await?;
            }
        }

        Ok(report.is_ready())
    }

    /// Look up the user's role for this assignment's module (if any).
    async fn role_for_user_in_module(
        &self,
        db: &DatabaseConnection,
        user_id: i64,
    ) -> Result<Option<ModuleRole>, DbErr> {
        let rec = UserModuleRoleEntity::find()
            .filter(UserModuleRoleCol::UserId.eq(user_id))
            .filter(UserModuleRoleCol::ModuleId.eq(self.module_id))
            .one(db)
            .await?;
        Ok(rec.map(|r| r.role))
    }

    /// Staff are any non-student roles.
    fn is_staff_role(role: &ModuleRole) -> bool {
        matches!(
            role,
            ModuleRole::Lecturer | ModuleRole::AssistantLecturer | ModuleRole::Tutor
        )
    }

    /// Load the assignment's ExecutionConfig from disk.
    /// Returns None if the file is missing or invalid.
    pub fn config(&self) -> Option<ExecutionConfig> {
        ExecutionConfig::get_execution_config(self.module_id, self.id).ok()
    }

    /// Whether attempt limits are enforced.
    pub fn limit_attempts(&self) -> bool {
        self.config()
            .map(|cfg| cfg.marking.limit_attempts)
            .unwrap_or(false)
    }

    /// Maximum attempts (default 10 if config missing).
    pub fn get_max_attempts(&self) -> u32 {
        self.config()
            .map(|cfg| cfg.marking.max_attempts)
            .unwrap_or(10)
    }

    /// Count the number of used attempts for a user on this assignment.
    ///
    /// Counts only **non-practice** and **non-ignored** submissions.
    pub async fn attempts_used_by_user(
        &self,
        db: &DatabaseConnection,
        user_id: i64,
    ) -> Result<u32, DbErr> {
        let count = SubmissionEntity::find()
            .filter(SubmissionCol::AssignmentId.eq(self.id))
            .filter(SubmissionCol::UserId.eq(user_id))
            .filter(SubmissionCol::IsPractice.eq(false))
            .filter(SubmissionCol::Ignored.eq(false))
            .count(db)
            .await?;
        Ok(count as u32)
    }

    /// Compute AttemptsSummary { used, max, remaining, limit_attempts }.
    pub async fn attempts_summary_for_user(
        &self,
        db: &DatabaseConnection,
        user_id: i64,
    ) -> Result<AttemptsSummary, DbErr> {
        let used = self.attempts_used_by_user(db, user_id).await?;
        let max = self.get_max_attempts();
        let limit_attempts = self.limit_attempts();

        let remaining = if limit_attempts {
            max.saturating_sub(used)
        } else {
            u32::MAX // effectively unlimited
        };

        Ok(AttemptsSummary {
            used,
            max,
            remaining,
            limit_attempts,
        })
    }

    /// Decide whether the user can submit another **non-practice** attempt.
    /// - Staff: always true (unlimited).
    /// - Students: obey attempt limits.
    pub async fn can_submit(&self, db: &DatabaseConnection, user_id: i64) -> Result<bool, DbErr> {
        if let Some(role) = self.role_for_user_in_module(db, user_id).await? {
            if Self::is_staff_role(&role) {
                return Ok(true); // staff are never limited
            }
        }
        // default to student rules if no role or role is student
        let summary = self.attempts_summary_for_user(db, user_id).await?;
        if !summary.limit_attempts {
            return Ok(true); // unlimited for students if not enforcing
        }
        Ok(summary.remaining > 0)
    }

    /// Whether practice submissions are enabled for this assignment (default false if config missing).
    pub fn allow_practice_submissions(&self) -> bool {
        self.config()
            .map(|cfg| cfg.marking.allow_practice_submissions)
            .unwrap_or(false)
    }

    /// Decide whether the user can submit given `is_practice`.
    ///
    ///     Staff (lecturer/assistant/tutor):
    ///   - Always allowed, practice or not.
    ///
    ///     Students:
    ///   - Practice requires `allow_practice_submissions == true`.
    ///   - Non-practice uses attempt-limit rules.
    pub async fn can_submit_for(
        &self,
        db: &DatabaseConnection,
        user_id: i64,
        is_practice: bool,
    ) -> Result<bool, DbErr> {
        if let Some(role) = self.role_for_user_in_module(db, user_id).await? {
            if Self::is_staff_role(&role) {
                return Ok(true); // staff ignore both practice flag and attempt limits
            }
        }

        // Student path
        if is_practice {
            return Ok(self.allow_practice_submissions());
        }
        self.can_submit(db, user_id).await
    }

    pub fn pass_mark(&self) -> u32 {
        self.config().map(|cfg| cfg.marking.pass_mark).unwrap_or(50) // default fallback
    }

    /// Automatically adjust an assignment's status based on the current time and its
    /// `available_from` / `due_date`, allowing only **adjacent** transitions.
    ///
    /// ### Allowed transitions (adjacent only)
    /// - `Ready  → Open`   when `now >= available_from`
    /// - `Open   → Closed` when `now >= due_date`
    /// - `Open   → Ready`  if `available_from` was moved into the future
    ///
    /// ### Not affected / terminal
    /// - `Setup`, `Archived`, and **`Closed`** are **never** auto-transitioned by this method.
    ///   (Closed is treated as terminal here; it will not auto re-open.)
    ///
    /// ### No jumps
    /// - The method only performs a **single, adjacent** step. It will **not** jump directly
    ///   from `Ready` → `Closed`, even if the current time is past the due date.
    ///
    /// ### Idempotency
    /// - If no adjacent change is needed, the method is a no-op.
    ///
    /// ### Time source
    /// - Uses `Utc::now()` to determine the current time.
    ///
    /// ### Returns
    /// - `Ok(Some(new_status))` if the status was updated.
    /// - `Ok(None)` if no change was needed.
    /// - `Err(DbErr)` if the database update fails.
    ///
    /// ### Typical usage
    /// - Call on read (e.g., when fetching an assignment) or from a periodic task to keep
    ///   statuses aligned with schedule changes.
    pub async fn auto_open_or_close(
        &self,
        db: &DatabaseConnection,
    ) -> Result<Option<Status>, DbErr> {
        let now = Utc::now();

        let desired = if now >= self.due_date {
            Status::Closed
        } else if now >= self.available_from {
            Status::Open
        } else {
            Status::Ready
        };

        // Only allow adjacent transitions among Ready/Open — Closed is terminal here.
        let allowed = matches!(
            (self.status.clone(), desired.clone()),
            (Status::Ready, Status::Open)
                | (Status::Open, Status::Closed)
                | (Status::Open, Status::Ready) // revert if available_from moved later
        );

        if allowed {
            let mut active = self.clone().into_active_model();
            active.status = Set(desired.clone());
            active.updated_at = Set(now);
            active.update(db).await?;
            Ok(Some(desired))
        } else {
            Ok(None)
        }
    }

    /// True if students must unlock (based on config.json).
    pub fn password_required_for_students(&self) -> bool {
        if let Some(cfg) = self.config() {
            cfg.security.password_enabled && cfg.security.password_pin.is_some()
        } else {
            false
        }
    }

    /// Verify a plaintext PIN against the PIN in the config.
    pub fn verify_password_from_config(&self, candidate: &str) -> bool {
        let Some(cfg) = self.config() else {
            return false;
        };
        let Some(ref pin) = cfg.security.password_pin else {
            return false;
        };
        // simple equality (you can swap to constant-time later if desired)
        candidate == pin
    }

    /// Short, stable tag that changes when the PIN changes.
    /// Used to invalidate old cookies immediately after rotation without DB.
    pub fn password_tag(&self) -> Option<String> {
        let cfg = self.config()?;
        let pin = cfg.security.password_pin.as_ref()?;
        let mut h = Sha256::new();
        h.update(pin.as_bytes());
        let hex = format!("{:x}", h.finalize());
        Some(hex[..16].to_string())
    }

    /// Whether the security cookie should be bound to the user id.
    pub fn bind_cookie_to_user(&self) -> bool {
        self.config()
            .map(|c| c.security.bind_cookie_to_user)
            .unwrap_or(true)
    }

    /// Whether the given client IP is allowed by the config’s CIDR allowlist.
    /// Empty allowlist => allow all. Missing/invalid config => allow.
    pub fn ip_allowed(&self, client_ip: IpAddr) -> bool {
        let Some(cfg) = self.config() else {
            return true;
        };
        if cfg.security.allowed_cidrs.is_empty() {
            return true;
        }
        for cidr in &cfg.security.allowed_cidrs {
            if let Ok(net) = cidr.parse::<IpNet>() {
                if net.contains(&client_ip) {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::module::ActiveModel as ModuleActiveModel;
    use crate::test_utils::setup_test_db;
    use chrono::{TimeZone, Utc};

    fn sample_dates() -> (DateTime<Utc>, DateTime<Utc>) {
        (
            Utc.with_ymd_and_hms(2025, 6, 1, 9, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2025, 6, 30, 17, 0, 0).unwrap(),
        )
    }

    #[tokio::test]
    async fn test_create_assignment() {
        let db = setup_test_db().await;
        let (from, due) = sample_dates();

        let module = ModuleActiveModel {
            code: Set("COS301".to_string()),
            year: Set(2025),
            description: Set(Some("Capstone Project".to_string())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("Failed to insert test module");

        let assignment = Model::create(
            &db,
            module.id,
            "Test Assignment",
            Some("Intro to Testing"),
            AssignmentType::Practical,
            from,
            due,
        )
        .await
        .unwrap();

        assert_eq!(assignment.module_id, module.id);
        assert_eq!(assignment.name, "Test Assignment");
        assert_eq!(assignment.status, Status::Setup); // status defaults to Setup
    }

    #[tokio::test]
    async fn test_edit_assignment() {
        let db = setup_test_db().await;
        let (from, due) = sample_dates();

        let module = ModuleActiveModel {
            code: Set("COS301".to_string()),
            year: Set(2025),
            description: Set(Some("Capstone Project".to_string())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("Failed to insert test module");

        let created = Model::create(
            &db,
            module.id,
            "Initial",
            Some("Initial Desc"),
            AssignmentType::Assignment,
            from,
            due,
        )
        .await
        .unwrap();

        let updated = Model::edit(
            &db,
            created.id,
            module.id,
            "Updated Name",
            Some("Updated Desc"),
            AssignmentType::Practical,
            from,
            due,
        )
        .await
        .unwrap();

        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.status, created.status); // status remains unchanged
    }

    #[tokio::test]
    async fn test_filter_assignments_by_query_and_sort() {
        let db = setup_test_db().await;
        let (from, due) = sample_dates();

        let module = ModuleActiveModel {
            code: Set("COS301".to_string()),
            year: Set(2025),
            description: Set(Some("Capstone Project".to_string())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("Failed to insert test module");

        Model::create(
            &db,
            module.id,
            "Rust Basics",
            Some("Learn Rust"),
            AssignmentType::Assignment,
            from,
            due,
        )
        .await
        .unwrap();
        Model::create(
            &db,
            module.id,
            "Advanced Rust",
            Some("Ownership and lifetimes"),
            AssignmentType::Assignment,
            from,
            due,
        )
        .await
        .unwrap();
        Model::create(
            &db,
            module.id,
            "Python Basics",
            Some("Learn Python"),
            AssignmentType::Assignment,
            from,
            due,
        )
        .await
        .unwrap();

        let rust_results = Model::filter(
            &db,
            module.id.try_into().unwrap(),
            10,
            Some("name".into()),
            Some("rust".into()),
        )
        .await
        .unwrap();

        assert_eq!(rust_results.len(), 2);
        assert!(
            rust_results
                .iter()
                .all(|a| a.name.to_lowercase().contains("rust"))
        );
    }

    #[tokio::test]
    async fn test_auto_transition_ready_to_open() {
        use crate::models::module::ActiveModel as ModuleActiveModel;
        use chrono::Duration;

        let db = crate::test_utils::setup_test_db().await;
        let now = Utc::now();

        // module
        let module = ModuleActiveModel {
            code: Set("TST101".to_string()),
            year: Set(2025),
            description: Set(Some("Test".to_string())),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        // Ready, available_from in the past, due_date in the future -> should OPEN
        let a = ActiveModel {
            module_id: Set(module.id),
            name: Set("A1".into()),
            description: Set(Some("desc".into())),
            assignment_type: Set(AssignmentType::Assignment),
            status: Set(Status::Ready),
            available_from: Set(now - Duration::hours(1)),
            due_date: Set(now + Duration::days(1)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let changed = a.auto_open_or_close(&db).await.unwrap();
        assert_eq!(changed, Some(Status::Open));

        let fresh = Entity::find_by_id(a.id).one(&db).await.unwrap().unwrap();
        assert_eq!(fresh.status, Status::Open);
    }

    #[tokio::test]
    async fn test_auto_transition_open_to_closed() {
        use crate::models::module::ActiveModel as ModuleActiveModel;
        use chrono::Duration;

        let db = crate::test_utils::setup_test_db().await;
        let now = Utc::now();

        let module = ModuleActiveModel {
            code: Set("TST102".to_string()),
            year: Set(2025),
            description: Set(Some("Test".to_string())),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        // Open, due_date in the past -> should CLOSE
        let a = ActiveModel {
            module_id: Set(module.id),
            name: Set("A2".into()),
            description: Set(Some("desc".into())),
            assignment_type: Set(AssignmentType::Assignment),
            status: Set(Status::Open),
            available_from: Set(now - Duration::days(2)),
            due_date: Set(now - Duration::minutes(1)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let changed = a.auto_open_or_close(&db).await.unwrap();
        assert_eq!(changed, Some(Status::Closed));

        let fresh = Entity::find_by_id(a.id).one(&db).await.unwrap().unwrap();
        assert_eq!(fresh.status, Status::Closed);
    }

    #[tokio::test]
    async fn test_auto_transition_open_back_to_ready_when_available_moved_later() {
        use crate::models::module::ActiveModel as ModuleActiveModel;
        use chrono::Duration;

        let db = crate::test_utils::setup_test_db().await;
        let now = Utc::now();

        let module = ModuleActiveModel {
            code: Set("TST104".to_string()),
            year: Set(2025),
            description: Set(Some("Test".to_string())),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        // Open, but available_from moved to the future -> should revert to READY
        let a = ActiveModel {
            module_id: Set(module.id),
            name: Set("A4".into()),
            description: Set(Some("desc".into())),
            assignment_type: Set(AssignmentType::Assignment),
            status: Set(Status::Open),
            available_from: Set(now + Duration::hours(2)), // moved later
            due_date: Set(now + Duration::days(1)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let changed = a.auto_open_or_close(&db).await.unwrap();
        assert_eq!(changed, Some(Status::Ready));

        let fresh = Entity::find_by_id(a.id).one(&db).await.unwrap().unwrap();
        assert_eq!(fresh.status, Status::Ready);
    }

    #[tokio::test]
    async fn test_auto_transition_no_change_for_setup_or_archived() {
        use crate::models::module::ActiveModel as ModuleActiveModel;
        use chrono::Duration;

        let db = crate::test_utils::setup_test_db().await;
        let now = Utc::now();

        let module = ModuleActiveModel {
            code: Set("TST105".to_string()),
            year: Set(2025),
            description: Set(Some("Test".to_string())),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        // Setup should not auto-transition
        let setup = ActiveModel {
            module_id: Set(module.id),
            name: Set("A5".into()),
            description: Set(Some("desc".into())),
            assignment_type: Set(AssignmentType::Assignment),
            status: Set(Status::Setup),
            available_from: Set(now - Duration::days(1)),
            due_date: Set(now + Duration::days(1)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let changed = setup.auto_open_or_close(&db).await.unwrap();
        assert_eq!(changed, None);
        let fresh = Entity::find_by_id(setup.id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fresh.status, Status::Setup);

        // Archived should not auto-transition
        let archived = ActiveModel {
            module_id: Set(module.id),
            name: Set("A6".into()),
            description: Set(Some("desc".into())),
            assignment_type: Set(AssignmentType::Assignment),
            status: Set(Status::Archived),
            available_from: Set(now - Duration::days(1)),
            due_date: Set(now - Duration::hours(1)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let changed = archived.auto_open_or_close(&db).await.unwrap();
        assert_eq!(changed, None);
        let fresh = Entity::find_by_id(archived.id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fresh.status, Status::Archived);
    }

    #[tokio::test]
    async fn test_auto_transition_does_not_jump_ready_to_closed() {
        use crate::models::module::ActiveModel as ModuleActiveModel;
        use chrono::Duration;

        let db = crate::test_utils::setup_test_db().await;
        let now = Utc::now();

        let module = ModuleActiveModel {
            code: Set("TST106".to_string()),
            year: Set(2025),
            description: Set(Some("Test".to_string())),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        // Ready but already past due_date -> should NOT jump directly to Closed
        let a = ActiveModel {
            module_id: Set(module.id),
            name: Set("A7".into()),
            description: Set(Some("desc".into())),
            assignment_type: Set(AssignmentType::Assignment),
            status: Set(Status::Ready),
            available_from: Set(now - Duration::days(2)),
            due_date: Set(now - Duration::hours(1)), // desired would be Closed
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let changed = a.auto_open_or_close(&db).await.unwrap();
        assert_eq!(changed, None); // no direct Ready -> Closed
        let fresh = Entity::find_by_id(a.id).one(&db).await.unwrap().unwrap();
        assert_eq!(fresh.status, Status::Ready);
    }
}
