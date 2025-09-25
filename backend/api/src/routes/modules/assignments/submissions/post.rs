use super::common::{MarkSummary, PlagiarismInfo, SubmissionDetailResponse};
use crate::services::email::EmailService;
use crate::{
    auth::AuthUser, response::ApiResponse, routes::modules::assignments::get::is_late,
    ws::modules::assignments::submissions::topics::submission_topic,
};
use axum::{
    Json,
    extract::{Extension, Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use code_runner;
use db::models::assignment_submission_output;
use db::models::assignment_task;
use db::models::user::Entity as UserEntity;
use db::models::{
    assignment::{Column as AssignmentColumn, Entity as AssignmentEntity},
    assignment_submission::{self, Model as AssignmentSubmissionModel},
};
use db::models::{assignment_memo_output, assignment_submission::SubmissionStatus};
use marker::MarkingJob;
use marker::comparators::{
    exact_comparator::ExactComparator, percentage_comparator::PercentageComparator,
    regex_comparator::RegexComparator,
};
use marker::error::MarkerError;
use marker::feedback::{
    ai_feedback::AiFeedback, auto_feedback::AutoFeedback, manual_feedback::ManualFeedback,
};
use md5;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, Order, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fs, path::PathBuf};
use tokio_util::bytes;
use util::paths::{
    assignment_dir, attempt_dir, mark_allocator_path as allocator_path, memo_output_dir,
    submission_report_path,
};
use util::paths::{storage_root as storage_root_path, submission_output_dir};
use util::{
    execution_config::{
        ExecutionConfig, {FeedbackScheme, MarkingScheme, SubmissionMode},
    },
    mark_allocator, scan_code_content,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct RemarkRequest {
    #[serde(default)]
    submission_ids: Option<Vec<i64>>,
    #[serde(default)]
    all: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ResubmitRequest {
    #[serde(default)]
    submission_ids: Option<Vec<i64>>,
    #[serde(default)]
    all: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct BulkOperationResponse {
    processed: usize,
    failed: Vec<FailedOperation>,
}

#[derive(Debug, Serialize)]
pub struct RemarkResponse {
    regraded: usize,
    failed: Vec<FailedOperation>,
}

#[derive(Debug, Serialize)]
pub struct ResubmitResponse {
    resubmitted: usize,
    failed: Vec<FailedOperation>,
}

#[derive(Debug, Serialize)]
pub struct FailedOperation {
    id: Option<i64>,
    error: String,
}

const ERR_LATE_NOT_ALLOWED: &str = "Late submissions are not allowed";

// ============================================================================
// Helper Functions
// ============================================================================

fn status_is_failed(status: &SubmissionStatus) -> bool {
    matches!(
        status,
        SubmissionStatus::FailedUpload
            | SubmissionStatus::FailedCompile
            | SubmissionStatus::FailedExecution
            | SubmissionStatus::FailedGrading
            | SubmissionStatus::FailedInternal
            | SubmissionStatus::FailedDisallowedCode
    )
}

fn within_late_window(
    submitted_at: chrono::DateTime<chrono::Utc>,
    due: chrono::DateTime<chrono::Utc>,
    window_minutes: u32,
) -> bool {
    if submitted_at <= due {
        return true;
    }
    let latest_ok = due + chrono::Duration::minutes(window_minutes as i64);
    submitted_at <= latest_ok
}

/// Returns (adjusted_earned, capped_to_opt)
fn cap_late_earned(earned: f64, total: f64, max_percent: f64) -> (f64, Option<f64>) {
    let cap = max_percent * total / 100.0;
    if earned > cap {
        (cap, Some(cap))
    } else {
        (earned, None)
    }
}

#[derive(Serialize)]
struct WsMark {
    earned: f64,
    total: f64,
}

async fn emit_submission_status(
    app: &AppState,
    module_id: i64,
    assignment_id: i64,
    user_id: i64,
    submission_id: i64,
    status: SubmissionStatus,
    message: Option<String>,
    mark: Option<WsMark>,
) {
    let topic = submission_topic(module_id, assignment_id, user_id);

    let mut payload = serde_json::json!({
        "event": "submission_status",
        "submission_id": submission_id,
        "status": status.to_string(),
        "ts": chrono::Utc::now().to_rfc3339(),
    });

    if status_is_failed(&status) {
        if let Some(msg) = message {
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("message".into(), serde_json::Value::String(msg));
            }
        }
    }

    if let Some(m) = mark {
        if let Some(obj) = payload.as_object_mut() {
            obj.insert(
                "mark".into(),
                serde_json::json!({ "earned": m.earned, "total": m.total }),
            );
        }
    }

    app.ws().broadcast(&topic, payload.to_string()).await;
}

/// Applies the appropriate comparator to a marking job based on the scheme
fn apply_comparator<'a>(marking_job: MarkingJob<'a>, scheme: &MarkingScheme) -> MarkingJob<'a> {
    match scheme {
        MarkingScheme::Exact => marking_job.with_comparator(ExactComparator),
        MarkingScheme::Percentage => marking_job.with_comparator(PercentageComparator),
        MarkingScheme::Regex => marking_job.with_comparator(RegexComparator),
    }
}

/// Applies the appropriate feedback to a marking job based on the scheme
fn apply_feedback<'a>(marking_job: MarkingJob<'a>, scheme: &FeedbackScheme) -> MarkingJob<'a> {
    match scheme {
        FeedbackScheme::Auto => marking_job.with_feedback(AutoFeedback),
        FeedbackScheme::Manual => marking_job.with_feedback(ManualFeedback),
        FeedbackScheme::Ai => marking_job.with_feedback(AiFeedback),
    }
}

/// Result type for disallowed code checking
#[derive(Debug)]
pub enum DisallowedCodeCheckResult {
    /// No disallowed code found - continue with normal processing
    Clean,
    /// Disallowed code found - should set mark to zero
    DisallowedFound(SubmissionDetailResponse),
    /// Error occurred during checking - continue with normal processing (best-effort)
    CheckFailed(String),
}

/// Builds a SubmissionDetailResponse for disallowed code submissions
fn build_disallowed_submission_response(
    submission: &AssignmentSubmissionModel,
    total_marks: f64,
    assignment: &db::models::assignment::Model,
) -> SubmissionDetailResponse {
    let now = Utc::now();
    SubmissionDetailResponse {
        id: submission.id,
        attempt: submission.attempt,
        filename: submission.filename.clone(),
        hash: submission.file_hash.clone(),
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
        mark: MarkSummary {
            earned: 0.0,
            total: total_marks,
        },
        is_practice: submission.is_practice,
        is_late: is_late(submission.created_at, assignment.due_date),
        ignored: submission.ignored,
        status: SubmissionStatus::FailedDisallowedCode.to_string(),
        tasks: vec![],
        code_coverage: None,
        user: None,
        plagiarism: PlagiarismInfo {
            flagged: false,
            similarity: 0.0,
            lines_matched: 0,
            description: "".to_string(),
        },
    }
}

/// Saves the submission report to disk
fn save_submission_report(
    response: &SubmissionDetailResponse,
    module_id: i64,
    assignment_id: i64,
    user_id: i64,
    attempt: i64,
) -> Result<(), String> {
    let report_path = submission_report_path(module_id, assignment_id, user_id, attempt);
    if let Some(parent) = report_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(response) {
        let _ = fs::write(&report_path, json);
    }
    Ok(())
}

/// Checks disallowed code for an existing submission by ID
///
/// This function loads an existing submission and checks if it contains disallowed code.
/// If disallowed code is found, it updates the submission status and returns the response.
///
/// # Arguments
/// * `submission_id` - ID of the existing submission to check
/// * `file_bytes` - The file bytes to scan for disallowed code
/// * `config` - The execution configuration containing disallowed patterns
/// * `db` - Database connection
/// * `assignment` - Assignment model for metadata
///
/// # Returns
/// * `DisallowedCodeCheckResult` indicating the scan result
pub async fn check_disallowed_code_existing(
    submission_id: i64,
    file_bytes: &[u8],
    config: &ExecutionConfig,
    db: &sea_orm::DatabaseConnection,
    assignment: &db::models::assignment::Model,
) -> DisallowedCodeCheckResult {
    println!(
        "Checking existing submission {} for disallowed code...",
        submission_id
    );

    // First, load the existing submission
    let submission = match assignment_submission::Entity::find_by_id(submission_id)
        .one(db)
        .await
    {
        Ok(Some(sub)) => sub,
        Ok(None) => {
            return DisallowedCodeCheckResult::CheckFailed(format!(
                "Submission {} not found",
                submission_id
            ));
        }
        Err(e) => {
            return DisallowedCodeCheckResult::CheckFailed(format!(
                "Database error loading submission: {}",
                e
            ));
        }
    };

    // Check if the file contains disallowed code
    match scan_code_content::contains_dissalowed_code(file_bytes, config) {
        Ok(true) => {
            // Load allocator for total marks
            let allocator =
                match mark_allocator::load_allocator(assignment.module_id, assignment.id) {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("Failed to load allocator: {}", e);
                        return DisallowedCodeCheckResult::CheckFailed(format!(
                            "Failed to load mark allocator: {}",
                            e
                        ));
                    }
                };

            // Update submission status to ignored and failed_disallowed_code
            let updated =
                match AssignmentSubmissionModel::set_ignored(db, submission.id, true).await {
                    Ok(u) => u,
                    Err(e) => {
                        eprintln!("Failed to set ignored flag: {:?}", e);
                        submission.clone()
                    }
                };

            let updated = match AssignmentSubmissionModel::update_status(
                db,
                updated.id,
                assignment_submission::SubmissionStatus::FailedDisallowedCode,
            )
            .await
            {
                Ok(u) => u,
                Err(e) => {
                    eprintln!("Failed to set failed_disallowed_code: {:?}", e);
                    updated
                }
            };

            // Build response using shared helper
            let response =
                build_disallowed_submission_response(&updated, allocator.total_value, assignment);

            // Save report using shared helper
            if let Err(e) = save_submission_report(
                &response,
                assignment.module_id,
                assignment.id,
                updated.user_id,
                updated.attempt,
            ) {
                eprintln!("Failed to save submission report: {}", e);
            }

            DisallowedCodeCheckResult::DisallowedFound(response)
        }
        Ok(false) => {
            if submission.ignored {
                if let Err(e) =
                    AssignmentSubmissionModel::set_ignored(db, submission.id, false).await
                {
                    eprintln!("Failed to unignore submission: {:?}", e);
                    return DisallowedCodeCheckResult::CheckFailed(format!(
                        "Failed to unignore submission: {}",
                        e
                    ));
                }
            }
            DisallowedCodeCheckResult::Clean
        }
        Err(e) => {
            eprintln!("Disallowed scan error: {}", e);
            DisallowedCodeCheckResult::CheckFailed(format!("Scan error: {}", e))
        }
    }
}

/// Checks disallowed code for a new submission and creates it if disallowed code is found
///
/// This function scans the file bytes for disallowed code. If found, it creates a new submission
/// with zero marks and FailedDisallowedCode status.
///
/// # Arguments
/// * `file_bytes` - The uploaded file bytes to scan
/// * `config` - The execution configuration containing disallowed patterns
/// * `db` - Database connection for saving the submission if disallowed code is found
/// * `assignment_id` - ID of the assignment
/// * `user_id` - ID of the user submitting
/// * `attempt` - Attempt number
/// * `is_practice` - Whether this is a practice submission
/// * `file_name` - Name of the uploaded file
/// * `file_hash` - Hash of the file content
/// * `assignment` - Assignment model for metadata
///
/// # Returns
/// * `DisallowedCodeCheckResult` indicating the scan result, with complete response if disallowed
pub async fn check_disallowed_code_new(
    file_bytes: &[u8],
    config: &ExecutionConfig,
    db: &sea_orm::DatabaseConnection,
    assignment_id: i64,
    user_id: i64,
    attempt: i64,
    is_practice: bool,
    file_name: &str,
    file_hash: &str,
    assignment: &db::models::assignment::Model,
) -> DisallowedCodeCheckResult {
    println!("Checking new submission for disallowed code...");

    match scan_code_content::contains_dissalowed_code(file_bytes, config) {
        Ok(true) => {
            // Load allocator for total marks
            let allocator =
                match mark_allocator::load_allocator(assignment.module_id, assignment_id) {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("Failed to load allocator: {}", e);
                        return DisallowedCodeCheckResult::CheckFailed(format!(
                            "Failed to load mark allocator: {}",
                            e
                        ));
                    }
                };

            // Save the submission with zero mark and total from allocator
            let saved = match AssignmentSubmissionModel::save_file(
                db,
                assignment_id,
                user_id,
                attempt,
                0.0,
                allocator.total_value,
                is_practice,
                file_name,
                file_hash,
                file_bytes,
            )
            .await
            {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Error saving disallowed submission: {:?}", e);
                    return DisallowedCodeCheckResult::CheckFailed(format!(
                        "Failed to save submission: {}",
                        e
                    ));
                }
            };

            // Update submission status to ignored and failed_disallowed_code
            let updated = match AssignmentSubmissionModel::set_ignored(db, saved.id, true).await {
                Ok(u) => u,
                Err(e) => {
                    eprintln!("Failed to set ignored flag: {:?}", e);
                    saved
                }
            };

            let updated = match AssignmentSubmissionModel::update_status(
                db,
                updated.id,
                assignment_submission::SubmissionStatus::FailedDisallowedCode,
            )
            .await
            {
                Ok(u) => u,
                Err(e) => {
                    eprintln!("Failed to set failed_disallowed_code: {:?}", e);
                    updated
                }
            };

            // Build response using shared helper
            let response =
                build_disallowed_submission_response(&updated, allocator.total_value, assignment);

            // Save report using shared helper
            if let Err(e) = save_submission_report(
                &response,
                assignment.module_id,
                assignment.id,
                updated.user_id,
                updated.attempt,
            ) {
                eprintln!("Failed to save submission report: {}", e);
            }

            DisallowedCodeCheckResult::DisallowedFound(response)
        }
        Ok(false) => DisallowedCodeCheckResult::Clean,
        Err(e) => {
            eprintln!("Disallowed scan error: {}", e);
            DisallowedCodeCheckResult::CheckFailed(format!("Scan error: {}", e))
        }
    }
}

/// Validates bulk operation request ensuring exactly one of submission_ids or all is provided
fn validate_bulk_request(
    submission_ids: &Option<Vec<i64>>,
    all: &Option<bool>,
) -> Result<(), &'static str> {
    match (submission_ids, all) {
        (Some(ids), None) if !ids.is_empty() => Ok(()),
        (None, Some(true)) => Ok(()),
        (Some(_), None) => Err("submission_ids cannot be empty"),
        _ => Err("Must provide exactly one of submission_ids or all=true"),
    }
}

/// Resolves target submission IDs based on request parameters
async fn resolve_submission_ids(
    submission_ids: Option<Vec<i64>>,
    all: Option<bool>,
    assignment_id: i64,
    db: &sea_orm::DatabaseConnection,
) -> Result<Vec<i64>, String> {
    match (submission_ids, all) {
        (Some(ids), _) if !ids.is_empty() => Ok(ids),
        (_, Some(true)) => AssignmentSubmissionModel::find_by_assignment(assignment_id, db)
            .await
            .map_err(|e| format!("Failed to fetch submissions: {}", e)),
        _ => Err("Must provide either submission_ids or all=true".to_string()),
    }
}

/// Loads assignment and validates it exists for the given module
async fn load_assignment(
    module_id: i64,
    assignment_id: i64,
    db: &sea_orm::DatabaseConnection,
) -> Result<db::models::assignment::Model, String> {
    AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id))
        .filter(AssignmentColumn::ModuleId.eq(module_id))
        .one(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Assignment not found".to_string())
}

/// Gets assignment file paths and configurations
fn get_assignment_paths(
    module_id: i64,
    assignment_id: i64,
) -> Result<(PathBuf, PathBuf, Vec<PathBuf>), String> {
    let base_path = assignment_dir(module_id, assignment_id);
    let mark_allocator_path = allocator_path(module_id, assignment_id);
    let memo_dir = memo_output_dir(module_id, assignment_id);

    let mut memo_outputs: Vec<_> = match fs::read_dir(&memo_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_file())
            .collect(),
        Err(_) => Vec::new(),
    };
    memo_outputs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    Ok((base_path, mark_allocator_path, memo_outputs))
}

/// Loads execution configuration for an assignment
fn get_execution_config(module_id: i64, assignment_id: i64) -> Result<ExecutionConfig, String> {
    ExecutionConfig::get_execution_config(module_id, assignment_id)
        .map_err(|_| "Failed to load execution config".to_string())
}

/// Validates file upload requirements
fn validate_file_upload(
    file_name: &Option<String>,
    file_bytes: &Option<bytes::Bytes>,
) -> Result<(String, bytes::Bytes), (StatusCode, Json<ApiResponse<SubmissionDetailResponse>>)> {
    let file_name = match file_name {
        Some(name) => name.clone(),
        None => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiResponse::<SubmissionDetailResponse>::error(
                    "No file provided",
                )),
            ));
        }
    };

    let file_bytes = match file_bytes {
        Some(bytes) => bytes.clone(),
        None => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiResponse::<SubmissionDetailResponse>::error(
                    "No file provided",
                )),
            ));
        }
    };

    let allowed_extensions = [".tgz", ".gz", ".tar", ".zip"];
    let file_extension = std::path::Path::new(&file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!(".{}", ext.to_lowercase()));

    if !file_extension
        .as_ref()
        .map_or(false, |ext| allowed_extensions.contains(&ext.as_str()))
    {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<SubmissionDetailResponse>::error(
                "Only .tgz, .gz, .tar, and .zip files are allowed",
            )),
        ));
    }

    if file_bytes.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<SubmissionDetailResponse>::error(
                "Empty file provided",
            )),
        ));
    }

    Ok((file_name, file_bytes))
}

/// Gets the next attempt number for a user's assignment
async fn get_next_attempt(
    assignment_id: i64,
    user_id: i64,
    db: &sea_orm::DatabaseConnection,
) -> Result<i64, String> {
    let prev_attempt = assignment_submission::Entity::find()
        .filter(assignment_submission::Column::AssignmentId.eq(assignment_id))
        .filter(assignment_submission::Column::UserId.eq(user_id))
        .order_by_desc(assignment_submission::Column::Attempt)
        .one(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .map(|s| s.attempt)
        .unwrap_or(0);

    Ok(prev_attempt + 1)
}

/// Core grading function that can be used for initial submissions, regrading, and resubmission
async fn grade_submission(
    submission: AssignmentSubmissionModel,
    assignment: &db::models::assignment::Model,
    _memo_outputs: &[std::path::PathBuf],
    config: &util::execution_config::ExecutionConfig,
    db: &sea_orm::DatabaseConnection,
    strict_mismatch_error: bool,
) -> Result<SubmissionDetailResponse, String> {
    if let Err(e) = AssignmentSubmissionModel::set_grading(db, submission.id).await {
        eprintln!("Failed to update submission status to grading: {:?}", e);
    }

    // Fetch tasks ordered by task_number
    let tasks = assignment_task::Entity::find()
        .filter(assignment_task::Column::AssignmentId.eq(assignment.id))
        .order_by(assignment_task::Column::TaskNumber, Order::Asc)
        .all(db)
        .await
        .map_err(|e| format!("Failed to load tasks: {}", e))?;

    let mut ordered_memo_paths: Vec<std::path::PathBuf> = Vec::new();
    let mut ordered_student_paths: Vec<std::path::PathBuf> = Vec::new();
    let mut db_pairing_ok = true;

    for task in tasks.iter().filter(|t| !t.code_coverage) {
        // Memo output for this task
        match assignment_memo_output::Entity::find()
            .filter(assignment_memo_output::Column::AssignmentId.eq(assignment.id))
            .filter(assignment_memo_output::Column::TaskId.eq(task.id))
            .one(db)
            .await
        {
            Ok(Some(mo)) => ordered_memo_paths.push(storage_root_path().join(&mo.path)),
            _ => {
                db_pairing_ok = false;
                break;
            }
        }

        // Student output for this task (for this submission). Pick newest that exists on disk.
        let outputs_res = assignment_submission_output::Entity::find()
            .filter(assignment_submission_output::Column::SubmissionId.eq(submission.id))
            .filter(assignment_submission_output::Column::TaskId.eq(task.id))
            .order_by_desc(assignment_submission_output::Column::UpdatedAt)
            .all(db)
            .await;
        match outputs_res {
            Ok(outputs) => {
                let mut found = None;
                for so in outputs {
                    let path = storage_root_path().join(&so.path);
                    if path.exists() {
                        found = Some(path);
                        break;
                    }
                }
                if let Some(p) = found {
                    ordered_student_paths.push(p);
                } else {
                    db_pairing_ok = false;
                    break;
                }
            }
            Err(_) => {
                db_pairing_ok = false;
                break;
            }
        }
    }

    // Fallback: if DB-based pairing failed, scan directory and pair by task_id
    if !db_pairing_ok {
        ordered_memo_paths.clear();
        ordered_student_paths.clear();

        let mut pairs: Vec<(i32, std::path::PathBuf, std::path::PathBuf)> = Vec::new();
        let out_dir = submission_output_dir(
            assignment.module_id,
            assignment.id,
            submission.user_id,
            submission.attempt,
        );
        // Collect memo files from disk (no DB rows required)
        let mut memo_files_disk: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(memo_entries) =
            std::fs::read_dir(memo_output_dir(assignment.module_id, assignment.id))
        {
            for me in memo_entries.flatten() {
                let mp = me.path();
                if mp.is_file() {
                    memo_files_disk.push(mp);
                }
            }
        }
        memo_files_disk.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        if let Ok(entries) = std::fs::read_dir(&out_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                        if ext.eq_ignore_ascii_case("txt") {
                            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                                if let Ok(output_id) = stem.parse::<i64>() {
                                    if let Ok(Some(output)) =
                                        assignment_submission_output::Entity::find_by_id(output_id)
                                            .one(db)
                                            .await
                                    {
                                        if let Ok(Some(task)) =
                                            assignment_task::Entity::find_by_id(output.task_id)
                                                .one(db)
                                                .await
                                        {
                                            if !task.code_coverage {
                                                // Map task_number to memo file index (task_number is 1-based)
                                                if let Some(mp) = memo_files_disk
                                                    .get((task.task_number.max(1) as usize) - 1)
                                                {
                                                    pairs.push((
                                                        task.task_number as i32,
                                                        mp.clone(),
                                                        p.clone(),
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        pairs.sort_by_key(|(tn, _, _)| *tn);
        for (_tn, mp, sp) in pairs {
            ordered_memo_paths.push(mp);
            ordered_student_paths.push(sp);
        }

        // If still unmatched, proceed with zero-mark fallback by leaving both lists empty.
        if ordered_memo_paths.is_empty()
            || ordered_student_paths.is_empty()
            || ordered_memo_paths.len() != ordered_student_paths.len()
        {
            if strict_mismatch_error {
                return Err(format!(
                    "memo_paths.len() != student_paths.len(): {} != {}",
                    ordered_memo_paths.len(),
                    ordered_student_paths.len()
                ));
            } else {
                ordered_memo_paths.clear();
                ordered_student_paths.clear();
            }
        }
    }

    // Load normalized allocator using your util
    let allocator = mark_allocator::load_allocator(assignment.module_id, assignment.id)
        .map_err(|e| format!("Failed to load mark allocator: {}", e))?;

    // Construct MarkingJob with allocator object
    let mut marking_job = MarkingJob::new(
        ordered_memo_paths,
        ordered_student_paths,
        allocator,
        config.clone(),
    );

    marking_job = apply_comparator(marking_job, &config.marking.marking_scheme);
    marking_job = apply_feedback(marking_job, &config.marking.feedback_scheme);

    let coverage_path = attempt_dir(
        assignment.module_id,
        assignment.id,
        submission.user_id,
        submission.attempt,
    )
    .join("coverage_report.json");
    if coverage_path.exists() {
        marking_job = marking_job.with_coverage(coverage_path);
    }

    let valgrind_path = attempt_dir(
        assignment.module_id,
        assignment.id,
        submission.user_id,
        submission.attempt,
    )
    .join("valgrind_report.json");
    if valgrind_path.exists() {
        marking_job = marking_job.with_valgrind(valgrind_path);
    }

    let mark_report = match marking_job.mark().await {
        Ok(report) => report,
        Err(e) => {
            eprintln!("MARKING ERROR: {:#?}", e);
            if let Err(status_err) = AssignmentSubmissionModel::set_failed(
                db,
                submission.id,
                assignment_submission::SubmissionStatus::FailedGrading,
            )
            .await
            {
                eprintln!(
                    "Failed to update submission status to failed_grading: {:?}",
                    status_err
                );
            }

            let error_msg = match e {
                MarkerError::InputMismatch(msg)
                | MarkerError::InvalidJson(msg)
                | MarkerError::MissingField(msg)
                | MarkerError::IoError(msg)
                | MarkerError::MissingTaskId(msg)
                | MarkerError::ParseOutputError(msg) => msg,
            };
            return Err(error_msg);
        }
    };

    let mut mark = MarkSummary {
        earned: mark_report.data.mark.earned,
        total: mark_report.data.mark.total,
    };

    // Apply late cap if applicable
    let is_late_now = submission.created_at > assignment.due_date;
    let mut _late_capped_to: Option<f64> = None;
    if is_late_now && config.marking.late.allow_late_submissions {
        if within_late_window(
            submission.created_at,
            assignment.due_date,
            config.marking.late.late_window_minutes,
        ) {
            let (adj, capped) = cap_late_earned(
                mark.earned,
                mark.total,
                config.marking.late.late_max_percent,
            );
            mark.earned = adj;
            _late_capped_to = capped;
        }
    }

    let tasks = serde_json::to_value(&mark_report.data.tasks)
        .unwrap_or_default()
        .as_array()
        .cloned()
        .unwrap_or_default();
    let code_coverage = mark_report
        .data
        .code_coverage
        .as_ref()
        .map(|cov| {
            let summary = cov
                .summary
                .as_ref()
                .map(|s| super::common::CodeCoverageSummary {
                    earned: s.earned,
                    total: s.total,
                    total_lines: s.total_lines as u32,
                    covered_lines: s.covered_lines as u32,
                    coverage_percent: s.coverage_percent,
                });
            let files: Vec<serde_json::Value> = cov
                .files
                .iter()
                .map(|f| {
                    serde_json::json!({
                        "path": f.path,
                        "earned": f.earned,
                        "total": f.total,
                    })
                })
                .collect();
            serde_json::json!({
                "summary": summary,
                "files": files,
            })
        })
        .and_then(|v| serde_json::from_value::<super::common::CodeCoverage>(v).ok());

    let mut active_model: assignment_submission::ActiveModel = submission.clone().into();
    active_model.earned = sea_orm::ActiveValue::Set(mark.earned);
    active_model.total = sea_orm::ActiveValue::Set(mark.total);
    active_model.status =
        sea_orm::ActiveValue::Set(assignment_submission::SubmissionStatus::Graded);
    assignment_submission::Entity::update(active_model)
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;

    let now = Utc::now();
    let resp = SubmissionDetailResponse {
        id: submission.id,
        attempt: submission.attempt,
        filename: submission.filename.clone(),
        hash: submission.file_hash.clone(),
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
        mark,
        is_practice: submission.is_practice,
        is_late: is_late(submission.created_at, assignment.due_date),
        ignored: submission.ignored,
        status: submission.status.to_string(),
        tasks,
        code_coverage,
        user: None, // just ignore this lol
        plagiarism: PlagiarismInfo {
            flagged: false,
            similarity: 0.0,
            lines_matched: 0,
            description: "".to_string(),
        },
    };

    let report_path = submission_report_path(
        assignment.module_id,
        assignment.id,
        submission.user_id,
        submission.attempt,
    );
    if let Some(parent) = report_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(&resp)
        .map_err(|_| "Failed to serialize submission report".to_string())?;
    fs::write(&report_path, json).map_err(|e| e.to_string())?;

    Ok(resp)
}

async fn process_submission_code(
    db: &sea_orm::DatabaseConnection,
    submission_id: i64,
    config: ExecutionConfig,
    module_id: i64,
    assignment_id: i64,
) -> Result<(), String> {
    match config.project.submission_mode {
        SubmissionMode::Manual => {
            code_runner::create_submission_outputs_for_all_tasks(db, submission_id)
                .await
                .map_err(|e| format!("Code runner failed: {}", e))
        }

        SubmissionMode::GATLAM => {
            let res = ai::run_ga_job(db, submission_id, config, module_id, assignment_id)
                .await
                .map_err(|e| format!("GATLAM failed: {}", e));

            if res.is_ok() {
                if let Some(sub) = assignment_submission::Entity::find_by_id(submission_id)
                    .one(db)
                    .await
                    .map_err(|e| e.to_string())?
                {
                    if let Some(user) = UserEntity::find_by_id(sub.user_id)
                        .one(db)
                        .await
                        .map_err(|e| e.to_string())?
                    {
                        let to_email = user.email.clone();
                        let display_name = user.email.clone();

                        if let Err(e) = EmailService::send_marking_done_email(
                            &to_email,
                            &display_name,
                            submission_id,
                            module_id,
                            assignment_id,
                        )
                        .await
                        {
                            tracing::warn!("send_marking_done_email failed: {}", e);
                        }
                    }
                }
            }
            res
        }

        SubmissionMode::CodeCoverage => {
            ai::run_coverage_ga_job(db, submission_id, &config, module_id, assignment_id)
                .await
                .map_err(|e| format!("Coverage GA failed: {}", e))
        }

        SubmissionMode::RNG => ai::run_rng_job(db, submission_id, &config)
            .await
            .map_err(|e| format!("RNG run failed: {}", e)),
    }
}

/// Clears the submission output directory
fn clear_submission_output(
    submission: &AssignmentSubmissionModel,
    module_id: i64,
    assignment_id: i64,
) -> Result<(), String> {
    let attempt = attempt_dir(
        module_id,
        assignment_id,
        submission.user_id,
        submission.attempt,
    );
    let output_dir = attempt.join("submission_output");
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir)
            .map_err(|e| format!("Failed to clear output directory: {}", e))?;
    }

    let report_path = submission_report_path(
        module_id,
        assignment_id,
        submission.user_id,
        submission.attempt,
    );
    if report_path.exists() {
        fs::remove_file(&report_path)
            .map_err(|e| format!("Failed to remove existing report: {}", e))?;
    }
    Ok(())
}

/// Executes bulk operation on submissions (remark or resubmit)
async fn execute_bulk_operation<F, Fut>(
    submission_ids: Vec<i64>,
    assignment_id: i64,
    db: &sea_orm::DatabaseConnection,
    operation: F,
) -> (usize, Vec<FailedOperation>)
where
    F: Fn(AssignmentSubmissionModel) -> Fut,
    Fut: std::future::Future<Output = Result<(), String>>,
{
    let mut processed = 0;
    let mut failed = Vec::new();

    for submission_id in submission_ids {
        let submission = match assignment_submission::Entity::find_by_id(submission_id)
            .one(db)
            .await
        {
            Ok(Some(sub)) => sub,
            Ok(None) => {
                failed.push(FailedOperation {
                    id: Some(submission_id),
                    error: "Submission not found".to_string(),
                });
                continue;
            }
            Err(e) => {
                failed.push(FailedOperation {
                    id: Some(submission_id),
                    error: format!("Database error: {}", e),
                });
                continue;
            }
        };

        if submission.assignment_id != assignment_id {
            failed.push(FailedOperation {
                id: Some(submission_id),
                error: "Submission does not belong to this assignment".to_string(),
            });
            continue;
        }

        match operation(submission).await {
            Ok(_) => processed += 1,
            Err(e) => failed.push(FailedOperation {
                id: Some(submission_id),
                error: e,
            }),
        }
    }

    (processed, failed)
}

#[derive(Debug, Deserialize)]
pub struct SubmitQuery {
    #[serde(default)]
    pub async_mode: Option<String>,
}

fn parse_bool_flag(v: Option<&str>) -> bool {
    match v.map(|s| s.to_ascii_lowercase()) {
        Some(ref s) if s == "true" || s == "1" || s == "on" || s == "yes" => true,
        _ => false,
    }
}

// ============================================================================
// Route Handlers
// ============================================================================

/// POST /api/modules/{module_id}/assignments/{assignment_id}/submissions
///
/// Submit an assignment file for grading. Accessible to authenticated students assigned to the module.
///
/// This endpoint accepts a multipart form upload containing the assignment file and optional flags.
/// The file is saved, graded, and a detailed grading report is returned. The grading process includes
/// code execution, mark allocation, and optional code coverage analysis.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to submit to
///
/// ### Request (multipart/form-data)
/// - `file` (required): The assignment file to upload (`.tgz`, `.gz`, `.tar`, or `.zip` only)
/// - `is_practice` (optional): If set to `true` or `1`, marks this as a practice submission
///
/// ### Example Request
/// ```bash
/// curl -X POST http://localhost:3000/api/modules/1/assignments/2/submissions \
///   -H "Authorization: Bearer <token>" \
///   -F "file=@solution.zip" \
///   -F "is_practice=true"
/// ```
///
/// ### Success Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "message": "Submission received and graded",
///   "data": {
///     "id": 123,
///     "attempt": 2,
///     "filename": "solution.zip",
///     "hash": "d41d8cd98f00b204e9800998ecf8427e",
///     "created_at": "2024-01-15T10:30:00Z",
///     "updated_at": "2024-01-15T10:30:00Z",
///     "mark": { "earned": 85, "total": 100 },
///     "is_practice": true,
///     "is_late": false,
///     "ignored": false,
///     "status": "graded",
///     "tasks": [ ... ],
///     "code_coverage": [ ... ],
///     "user": null,
///     "plagiarism": {
///         "flagged": false,
///         "similarity": 0.0,
///         "lines_matched": 0,
///         "description": ""
///     },
///   }
/// }
/// ```
///
/// ### Error Responses
///
/// **404 Not Found** - Assignment not found
/// ```json
/// { "success": false, "message": "Assignment not found" }
/// ```
///
/// **422 Unprocessable Entity** - File missing, invalid, or empty
/// ```json
/// { "success": false, "message": "No file provided" }
/// ```
/// or
/// ```json
/// { "success": false, "message": "Only .tgz, .gz, .tar, and .zip files are allowed" }
/// ```
/// or
/// ```json
/// { "success": false, "message": "Empty file provided" }
/// ```
///
/// **500 Internal Server Error** - Grading or system error
/// ```json
/// { "success": false, "message": "Failed to save submission" }
/// ```
/// or
/// ```json
/// { "success": false, "message": "Failed to run code for submission" }
/// ```
/// or
/// ```json
/// { "success": false, "message": "Failed to load mark allocator" }
/// ```
/// or
/// ```json
/// { "success": false, "message": "Failed to run code for submission" }
/// ```
///
/// ### Side Effects
/// - Saves the uploaded file and generated outputs to disk
/// - Triggers code execution and marking
/// - Saves a copy of the grading report as `submission_report.json` in the attempt folder
///
/// ### Notes
/// - Each submission increments the attempt number for the user/assignment
/// - Only one file per submission is accepted
/// - Practice submissions are marked and reported but may not count toward final grade
/// - The returned report includes detailed per-task grading and code coverage if available
/// - The endpoint is restricted to authenticated students assigned to the module
/// - All errors are returned in a consistent JSON format
// replace the entire submit_assignment with this version
// replace the entire submit_assignment with this version
pub async fn submit_assignment(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Query(q): Query<SubmitQuery>, // async_mode from query string
    Extension(AuthUser(claims)): Extension<AuthUser>,
    mut multipart: Multipart,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let db = app_state.db();
    let async_mode = parse_bool_flag(q.async_mode.as_deref());

    // ---- load assignment ----
    let assignment = match load_assignment(module_id, assignment_id, db).await {
        Ok(assignment) => assignment,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<serde_json::Value>::error(
                    "Assignment not found",
                )),
            );
        }
    };

    // ---- parse multipart ----
    let mut is_practice = false;
    let mut attests_ownership = false;
    let mut file_name: Option<String> = None;
    let mut file_bytes: Option<bytes::Bytes> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        match field.name() {
            Some("file") => {
                file_name = field.file_name().map(|s| s.to_string());
                file_bytes = Some(field.bytes().await.unwrap_or_default());
            }
            Some("is_practice") => {
                let v = field
                    .text()
                    .await
                    .unwrap_or_default()
                    .trim()
                    .to_ascii_lowercase();
                is_practice = v == "true" || v == "1" || v == "on";
            }
            Some("attests_ownership") => {
                let v = field
                    .text()
                    .await
                    .unwrap_or_default()
                    .trim()
                    .to_ascii_lowercase();
                attests_ownership = v == "true" || v == "1" || v == "on";
            }
            _ => {}
        }
    }

    // ---- preconditions / validation ----
    if !attests_ownership {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<serde_json::Value>::error(
                "You must confirm the ownership attestation before submitting",
            )),
        );
    }

    // use shared helper for file presence + extension + non-empty checks
    let (file_name, file_bytes) = match validate_file_upload(&file_name, &file_bytes) {
        Ok(v) => v,
        Err((status, Json(err))) => {
            // map typed error to Value response, reusing the same message
            return (
                status,
                Json(ApiResponse::<serde_json::Value>::error(&err.message)),
            );
        }
    };

    // size limit (mirror UI 50MB)
    const MAX_SIZE_MB: usize = 50;
    if file_bytes.len() > MAX_SIZE_MB * 1024 * 1024 {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(ApiResponse::<serde_json::Value>::error(
                "File too large. Max size is 50 MB",
            )),
        );
    }

    // policy check
    match assignment.can_submit_for(db, claims.sub, is_practice).await {
        Ok(true) => {}
        Ok(false) => {
            let msg = if is_practice {
                "Practice submissions are disabled for this assignment.".to_string()
            } else {
                match assignment.attempts_summary_for_user(db, claims.sub).await {
                    Ok(summary) => {
                        if summary.limit_attempts {
                            format!(
                                "Maximum attempts reached: used {} of {} (remaining {}).",
                                summary.used, summary.max, summary.remaining
                            )
                        } else {
                            "Submission attempts not allowed by policy.".to_string()
                        }
                    }
                    Err(_) => "Submission attempts not allowed by policy.".to_string(),
                }
            };
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<serde_json::Value>::error(&msg)),
            );
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<serde_json::Value>::error(
                    "Failed to evaluate submission attempts",
                )),
            );
        }
    }

    // load execution config
    let config = match get_execution_config(module_id, assignment_id) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<serde_json::Value>::error(&e)),
            );
        }
    };

    // attempt/hash
    let file_hash = format!("{:x}", md5::compute(&file_bytes));
    let attempt = match get_next_attempt(assignment_id, claims.sub, db).await {
        Ok(a) => a,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<serde_json::Value>::error(
                    "Failed to determine attempt number",
                )),
            );
        }
    };

    // disallowed code scan
    match check_disallowed_code_new(
        &file_bytes,
        &config,
        db,
        assignment_id,
        claims.sub,
        attempt,
        is_practice,
        &file_name,
        &file_hash,
        &assignment,
    )
    .await
    {
        DisallowedCodeCheckResult::Clean => {}
        DisallowedCodeCheckResult::DisallowedFound(response) => {
            // WS: failed_upload + zero mark
            emit_submission_status(
                &app_state,
                module_id,
                assignment_id,
                claims.sub,
                response.id,
                SubmissionStatus::FailedUpload,
                Some("Disallowed code patterns detected".into()),
                Some(WsMark {
                    earned: 0.0,
                    total: response.mark.total,
                }),
            )
            .await;

            let body = json!({
                "id": response.id,
                "status": SubmissionStatus::FailedUpload.to_string(),
                "attempt": response.attempt,
                "is_practice": response.is_practice,
                "filename": response.filename,
                "hash": response.hash,
                "created_at": response.created_at,
            });
            return (
                StatusCode::ACCEPTED,
                Json(ApiResponse::success(
                    body,
                    "Submission rejected: disallowed code patterns detected (marked as 0)",
                )),
            );
        }
        DisallowedCodeCheckResult::CheckFailed(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<serde_json::Value>::error(
                    "Failed to scan submission for disallowed code patterns",
                )),
            );
        }
    }

    // save submission row + file
    let submission = match AssignmentSubmissionModel::save_file(
        db,
        assignment_id,
        claims.sub,
        attempt,
        0.0,
        0.0,
        is_practice,
        &file_name,
        &file_hash,
        &file_bytes,
    )
    .await
    {
        Ok(m) => m,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<serde_json::Value>::error(
                    "Failed to save submission",
                )),
            );
        }
    };

    // emit QUEUED immediately for both modes
    emit_submission_status(
        &app_state,
        module_id,
        assignment_id,
        claims.sub,
        submission.id,
        SubmissionStatus::Queued,
        None,
        None,
    )
    .await;

    // ====== BRANCH: async vs blocking ======
    if async_mode {
        // ---------- ASYNC: spawn and return ACK ----------
        let db_bg = db.clone();
        let app_state_bg = app_state.clone();
        let assignment_bg = assignment.clone();
        let config_bg = config.clone();
        let submission_bg = submission.clone();

        tokio::spawn(async move {
            let _ = run_submission_pipeline(
                &db_bg,
                &app_state_bg,
                &assignment_bg,
                &config_bg,
                &submission_bg,
                module_id,
                assignment_id,
                submission_bg.user_id,
            )
            .await;
        });

        let ack = json!({
            "id": submission.id,
            "status": SubmissionStatus::Queued.to_string(),
            "attempt": submission.attempt,
            "is_practice": submission.is_practice,
            "filename": submission.filename,
            "hash": submission.file_hash,
            "created_at": Utc::now().to_rfc3339(),
        });

        return (
            StatusCode::ACCEPTED,
            Json(ApiResponse::success(ack, "Submission queued")),
        );
    }

    // ---------- BLOCKING: run pipeline inline and return full detail ----------
    match run_submission_pipeline(
        db,
        &app_state,
        &assignment,
        &config,
        &submission,
        module_id,
        assignment_id,
        claims.sub,
    )
    .await
    {
        Ok(resp) => {
            let body = serde_json::to_value(&resp).unwrap_or_else(|_| json!({}));
            (
                StatusCode::OK,
                Json(ApiResponse::success(body, "Submission received and graded")),
            )
        }
        Err(err_msg) => {
            let status = if err_msg == ERR_LATE_NOT_ALLOWED {
                StatusCode::UNPROCESSABLE_ENTITY
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(ApiResponse::<serde_json::Value>::error(&err_msg)),
            )
        }
    }
}

/// Runs the full submission pipeline and emits WS status events along the way.
/// Returns Ok(SubmissionDetailResponse) on success; Err(String) on failure.
/// NOTE: this function *already* emits terminal WS statuses on both success and failure.
async fn run_submission_pipeline(
    db: &DatabaseConnection,
    app: &AppState,
    assignment: &db::models::assignment::Model,
    config: &ExecutionConfig,
    submission: &AssignmentSubmissionModel,
    module_id: i64,
    assignment_id: i64,
    user_id: i64,
) -> Result<SubmissionDetailResponse, String> {
    // --- Late acceptance gate ---
    let submitted_at = submission.created_at;
    let due = assignment.due_date;
    let is_late_now = submitted_at > due;
    if is_late_now {
        let late = &config.marking.late;
        if !late.allow_late_submissions
            || !within_late_window(submitted_at, due, late.late_window_minutes)
        {
            let _ = AssignmentSubmissionModel::set_failed(
                db,
                submission.id,
                assignment_submission::SubmissionStatus::FailedUpload,
            )
            .await;

            emit_submission_status(
                app,
                module_id,
                assignment_id,
                user_id,
                submission.id,
                SubmissionStatus::FailedUpload,
                Some("Late submissions are not allowed for this assignment".into()),
                None,
            )
            .await;

            // use the constant here:
            return Err(ERR_LATE_NOT_ALLOWED.to_string());
        }
    }

    // running
    if AssignmentSubmissionModel::set_running(db, submission.id)
        .await
        .is_ok()
    {
        emit_submission_status(
            app,
            module_id,
            assignment_id,
            user_id,
            submission.id,
            SubmissionStatus::Running,
            None,
            None,
        )
        .await;
    }

    // execute
    if let Err(exec_err) =
        process_submission_code(db, submission.id, config.clone(), module_id, assignment_id).await
    {
        let _ = AssignmentSubmissionModel::set_failed(
            db,
            submission.id,
            assignment_submission::SubmissionStatus::FailedExecution,
        )
        .await;

        emit_submission_status(
            app,
            module_id,
            assignment_id,
            user_id,
            submission.id,
            SubmissionStatus::FailedExecution,
            Some(exec_err.to_string()),
            None,
        )
        .await;

        // return Err("Failed to run code for submission".to_string());
        return Err(format!("Failed to run code for submission: {}", exec_err));
    }

    // grading start
    emit_submission_status(
        app,
        module_id,
        assignment_id,
        user_id,
        submission.id,
        SubmissionStatus::Grading,
        None,
        None,
    )
    .await;

    // paths
    let (_, _, memo_outputs) = match get_assignment_paths(assignment.module_id, assignment.id) {
        Ok(paths) => paths,
        Err(e) => {
            let _ = AssignmentSubmissionModel::set_failed(
                db,
                submission.id,
                assignment_submission::SubmissionStatus::FailedInternal,
            )
            .await;

            emit_submission_status(
                app,
                module_id,
                assignment_id,
                user_id,
                submission.id,
                SubmissionStatus::FailedInternal,
                Some(e.clone()),
                None,
            )
            .await;

            return Err("Failed to load mark allocator".to_string());
        }
    };

    // grade
    match grade_submission(
        submission.clone(),
        assignment,
        &memo_outputs,
        config,
        db,
        false,
    )
    .await
    {
        Ok(resp) => {
            emit_submission_status(
                app,
                module_id,
                assignment_id,
                user_id,
                submission.id,
                SubmissionStatus::Graded,
                None,
                Some(WsMark {
                    earned: resp.mark.earned,
                    total: resp.mark.total,
                }),
            )
            .await;

            Ok(resp)
        }
        Err(e) => {
            let _ = AssignmentSubmissionModel::set_failed(
                db,
                submission.id,
                assignment_submission::SubmissionStatus::FailedGrading,
            )
            .await;

            emit_submission_status(
                app,
                module_id,
                assignment_id,
                user_id,
                submission.id,
                SubmissionStatus::FailedGrading,
                Some(e.clone()),
                None,
            )
            .await;

            Err("Failed to run code for submission".to_string())
        }
    }
}

/// POST /api/modules/{module_id}/assignments/{assignment_id}/submissions/remark
///
/// Regrade (remark) assignment submissions. Accessible to lecturers, assistant lecturers, and admins.
///
/// This endpoint allows authorized users to re-run marking logic on either:
/// - Specific submissions (via `submission_ids`)
/// - All submissions in an assignment (via `all: true`)
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment containing the submissions
///
/// ### Request Body
/// Either:
/// ```json
/// { "submission_ids": [123, 124, 125] }
/// ```
/// or
/// ```json
/// { "all": true }
/// ```
///
/// ### Success Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "message": "Regraded 3/4 submissions",
///   "data": {
///     "regraded": 3,
///     "failed": [
///       { "id": 125, "error": "Submission not found" }
///     ]
///   }
/// }
/// ```
///
/// ### Error Responses
///
/// **400 Bad Request** - Invalid request parameters
/// ```json
/// { "success": false, "message": "Must provide either submission_ids or all=true" }
/// ```
///
/// **403 Forbidden** - User not authorized for operation
/// ```json
/// { "success": false, "message": "Not authorized to remark submissions" }
/// ```
///
/// **404 Not Found** - Assignment not found
/// ```json
/// { "success": false, "message": "Assignment not found" }
/// ```
///
/// **500 Internal Server Error** - Regrading failure
/// ```json
/// { "success": false, "message": "Failed to load mark allocator" }
/// ```
pub async fn remark_submissions(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(req): Json<RemarkRequest>,
) -> impl IntoResponse {
    let db = app_state.db();

    if let Err(e) = validate_bulk_request(&req.submission_ids, &req.all) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<RemarkResponse>::error(e)),
        );
    }

    let assignment = match load_assignment(module_id, assignment_id, db).await {
        Ok(assignment) => assignment,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<RemarkResponse>::error("Assignment not found")),
            );
        }
    };

    let submission_ids =
        match resolve_submission_ids(req.submission_ids, req.all, assignment_id, db).await {
            Ok(ids) => ids,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<RemarkResponse>::error(e)),
                );
            }
        };

    let (_, _, memo_outputs) = match get_assignment_paths(assignment.module_id, assignment.id) {
        Ok(paths) => paths,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<RemarkResponse>::error(e)),
            );
        }
    };

    let config = match get_execution_config(module_id, assignment_id) {
        Ok(config) => config,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<RemarkResponse>::error(e)),
            );
        }
    };

    let (regraded, failed) =
        execute_bulk_operation(submission_ids.clone(), assignment_id, db, |submission| {
            let assignment = assignment.clone();
            let memo_outputs = memo_outputs.clone();
            let config = config.clone();
            async move {
                // Extract extension from submission filename
                let ext = std::path::PathBuf::from(&submission.filename)
                    .extension()
                    .map(|e| e.to_string_lossy().to_string());

                if let Ok(file_bytes) = std::fs::read(util::paths::submission_file_path(
                    assignment.module_id,
                    assignment.id,
                    submission.user_id,
                    submission.attempt,
                    submission.id,
                    ext.as_deref(),
                )) {
                    match check_disallowed_code_existing(
                        submission.id,
                        &file_bytes,
                        &config,
                        db,
                        &assignment,
                    )
                    .await
                    {
                        DisallowedCodeCheckResult::Clean => {
                            // Continue with normal processing
                        }
                        DisallowedCodeCheckResult::DisallowedFound(_response) => {
                            return Ok(());
                        }
                        DisallowedCodeCheckResult::CheckFailed(e) => {
                            eprintln!("Disallowed code check failed: {}", e);
                            return Err("Failed to scan submission for disallowed code patterns"
                                .to_string());
                        }
                    }
                } else {
                    return Err("Failed to read submission file from disk".to_string());
                }

                grade_submission(submission, &assignment, &memo_outputs, &config, db, true)
                    .await
                    .map(|_| ())
            }
        })
        .await;

    let response = RemarkResponse { regraded, failed };
    let message = format!("Regraded {}/{} submissions", regraded, submission_ids.len());

    (
        StatusCode::OK,
        Json(ApiResponse::success(response, &message)),
    )
}

/// POST /api/modules/{module_id}/assignments/{assignment_id}/submissions/resubmit
///
/// Reprocess assignment submissions using the latest marking pipeline. Accessible to admins, module lecturers, and assistant lecturers.
///
/// This endpoint allows authorized users to rerun the entire submission pipeline (code execution + marking) on either:
/// - Specific submissions (via `submission_ids`)
/// - All submissions in an assignment (via `all: true`)
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment containing the submissions
///
/// ### Request Body
/// Either:
/// ```json
/// { "submission_ids": [123, 124, 125] }
/// ```
/// or
/// ```json
/// { "all": true }
/// ```
///
/// ### Success Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "message": "Resubmitted 3/4 submissions",
///   "data": {
///     "resubmitted": 3,
///     "failed": [
///       { "id": 125, "error": "Submission not found" }
///     ]
///   }
/// }
/// ```
///
/// ### Error Responses
///
/// **400 Bad Request** - Invalid request parameters
/// ```json
/// { "success": false, "message": "Must provide exactly one of submission_ids or all=true" }
/// ```
/// or
/// ```json
/// { "success": false, "message": "submission_ids cannot be empty" }
/// ```
///
/// **403 Forbidden** - User not authorized for operation
/// ```json
/// { "success": false, "message": "Not authorized to resubmit submissions" }
/// ```
///
/// **404 Not Found** - Assignment not found
/// ```json
/// { "success": false, "message": "Assignment not found" }
/// ```
///
/// **500 Internal Server Error** - Resubmission failure
/// ```json
/// { "success": false, "message": "Failed to load mark allocator" }
/// ```
/// or
/// ```json
/// { "success": false, "message": "Failed to run code for submission" }
/// ```
///
/// ### Side Effects
/// - Re-executes code for all target submissions
/// - Regenerates marking reports and saves updated `submission_report.json` files
/// - Updates submission status transitions as applicable
///
/// ### Notes
/// - Resubmission reruns the entire pipeline: code execution → marking → report generation
/// - This differs from remark which only reruns the marking phase
/// - The endpoint is restricted to admin, module lecturer, and assistant lecturer roles
/// - All errors are returned in a consistent JSON format with per-submission failure details
pub async fn resubmit_submissions(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Extension(AuthUser(_claims)): Extension<AuthUser>,
    Json(req): Json<ResubmitRequest>,
) -> impl IntoResponse {
    let db = app_state.db();

    if let Err(e) = validate_bulk_request(&req.submission_ids, &req.all) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<ResubmitResponse>::error(e)),
        );
    }

    let assignment = match load_assignment(module_id, assignment_id, db).await {
        Ok(assignment) => assignment,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<ResubmitResponse>::error(
                    "Assignment not found",
                )),
            );
        }
    };

    let submission_ids =
        match resolve_submission_ids(req.submission_ids, req.all, assignment_id, db).await {
            Ok(ids) => ids,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<ResubmitResponse>::error(e)),
                );
            }
        };

    let (_, _, memo_outputs) = match get_assignment_paths(assignment.module_id, assignment.id) {
        Ok(paths) => paths,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<ResubmitResponse>::error(e)),
            );
        }
    };

    let config = match get_execution_config(module_id, assignment_id) {
        Ok(config) => config,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<ResubmitResponse>::error(e)),
            );
        }
    };

    // Run resubmissions concurrently per submission, with bounded concurrency
    use std::sync::Arc;
    use tokio::sync::Semaphore;
    use tokio::task::JoinSet;

    let max_concurrency = std::cmp::max(
        1,
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            / 2,
    );
    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let mut join_set = JoinSet::new();

    for sid in submission_ids.clone() {
        let db = db.clone();
        let assignment = assignment.clone();
        let memo_outputs = memo_outputs.clone();
        let config = config.clone();
        let sem = semaphore.clone();

        join_set.spawn(async move {
            let _permit = sem.acquire_owned().await.ok();

            // Fetch submission and validate
            let submission = match assignment_submission::Entity::find_by_id(sid)
                .one(&db)
                .await
            {
                Ok(Some(s)) => s,
                Ok(None) => return (sid, Err("Submission not found".to_string())),
                Err(e) => return (sid, Err(format!("Database error: {}", e))),
            };
            if submission.assignment_id != assignment.id {
                return (
                    sid,
                    Err("Submission does not belong to this assignment".to_string()),
                );
            }

            // Extract extension from submission filename
            let ext = std::path::PathBuf::from(&submission.filename)
                .extension()
                .map(|e| e.to_string_lossy().to_string());

            if let Ok(file_bytes) = std::fs::read(util::paths::submission_file_path(
                assignment.module_id,
                assignment.id,
                submission.user_id,
                submission.attempt,
                submission.id,
                ext.as_deref(),
            )) {
                match check_disallowed_code_existing(
                    submission.id,
                    &file_bytes,
                    &config,
                    &db,
                    &assignment,
                )
                .await
                {
                    DisallowedCodeCheckResult::Clean => {
                        // Continue with normal processing
                    }
                    DisallowedCodeCheckResult::DisallowedFound(_response) => {
                        return (sid, Ok(()));
                    }
                    DisallowedCodeCheckResult::CheckFailed(e) => {
                        eprintln!("Disallowed code check failed: {}", e);
                        return (
                            sid,
                            Err("Failed to scan submission for disallowed code patterns"
                                .to_string()),
                        );
                    }
                }
            } else {
                return (
                    sid,
                    Err("Failed to read submission file from disk".to_string()),
                );
            }

            if let Err(e) =
                clear_submission_output(&submission, assignment.module_id, assignment.id)
            {
                return (sid, Err(e));
            }

            if let Err(e) = process_submission_code(
                &db,
                submission.id,
                config.clone(),
                module_id,
                assignment_id,
            )
            .await
            {
                return (
                    sid,
                    Err(format!("Failed to run code for submission: {}", e)),
                );
            }

            match grade_submission(submission, &assignment, &memo_outputs, &config, &db, true).await
            {
                Ok(_) => (sid, Ok(())),
                Err(e) => (sid, Err(e)),
            }
        });
    }

    let mut resubmitted = 0usize;
    let mut failed: Vec<FailedOperation> = Vec::new();
    while let Some(res) = join_set.join_next().await {
        match res {
            Ok((_sid, Ok(()))) => resubmitted += 1,
            Ok((sid, Err(e))) => failed.push(FailedOperation {
                id: Some(sid),
                error: e,
            }),
            Err(e) => failed.push(FailedOperation {
                id: None,
                error: format!("Join error: {}", e),
            }),
        }
    }

    let response = ResubmitResponse {
        resubmitted,
        failed,
    };
    let message = format!(
        "Resubmitted {}/{} submissions",
        resubmitted,
        submission_ids.len()
    );

    (
        StatusCode::OK,
        Json(ApiResponse::success(response, &message)),
    )
}
