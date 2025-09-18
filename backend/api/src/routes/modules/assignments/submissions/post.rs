use std::{fs, path::PathBuf};

use super::common::{MarkSummary, SubmissionDetailResponse};
use crate::{auth::AuthUser, response::ApiResponse, routes::modules::assignments::get::is_late};
use axum::{
    Json,
    extract::{Extension, Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use code_runner;
use db::models::assignment_memo_output;
use db::models::assignment_task;
use db::models::{
    assignment::{Column as AssignmentColumn, Entity as AssignmentEntity},
    assignment_submission::{self, Model as AssignmentSubmissionModel},
    assignment_submission_output::Entity as AssignmentSubmissionOutputModel,
    assignment_task::Entity as AssignmentTaskModel,
};
use marker::MarkingJob;
use marker::comparators::{exact_comparator::ExactComparator, percentage_comparator::PercentageComparator, regex_comparator::RegexComparator};
use marker::feedback::{auto_feedback::AutoFeedback, manual_feedback::ManualFeedback, ai_feedback::AiFeedback};
use md5;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use tokio_util::bytes;
use util::{mark_allocator::mark_allocator::TaskInfo, paths::storage_root};
use util::{
    execution_config::{ExecutionConfig, execution_config::{SubmissionMode, MarkingScheme, FeedbackScheme}},
    mark_allocator::mark_allocator::generate_allocator,
    mark_allocator::mark_allocator::load_allocator,
    scan_code_content::scan_code_content,
    state::AppState,
};
use util::paths::{
    assignment_dir,
    memo_output_dir,
    mark_allocator_path as allocator_path,
    submission_output_dir,
    submission_report_path,
    attempt_dir,
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

// ============================================================================
// Helper Functions
// ============================================================================

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

/// Best-effort scan for disallowed code; logs errors and returns false on failure
fn scan_disallowed_best_effort<P: AsRef<std::path::Path>>(
    path: P,
    config: &ExecutionConfig,
) -> bool {
    match scan_code_content::contains_dissalowed_code(path, config) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Disallowed scan error: {}", e);
            false
        }
    }
}

/// Set earned=0 in DB and update the submission report to reflect the zero mark.
async fn enforce_zero_mark(
    module_id: i64,
    assignment_id: i64,
    submission: &AssignmentSubmissionModel,
    total: i64,
    tasks: Option<&Vec<serde_json::Value>>,
    db: &sea_orm::DatabaseConnection,
) -> Result<(), String> {
    // Update DB mark to 0
    if let Ok(Some(existing)) = assignment_submission::Entity::find_by_id(submission.id).one(db).await {
        let mut am: assignment_submission::ActiveModel = existing.into();
        am.earned = sea_orm::ActiveValue::Set(0);
        am.updated_at = sea_orm::ActiveValue::Set(Utc::now());
        if let Err(e) = assignment_submission::Entity::update(am).exec(db).await {
            eprintln!("Failed to zero mark in DB: {}", e);
        }
    }

    // Update report JSON atomically
    let new_mark = MarkSummary { earned: 0, total };
    update_submission_report_marks(
        module_id,
        assignment_id,
        submission,
        &new_mark,
        tasks,
    )
    .await
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

/// Loads the mark allocator for an assignment
async fn load_assignment_allocator(module_id: i64, assignment_id: i64) -> Result<(), String> {
    load_allocator(module_id, assignment_id)
        .await
        .map(|_| ())
        .map_err(|_| "Failed to load mark allocator".to_string())
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
    memo_outputs: &[std::path::PathBuf],
    mark_allocator_path: &std::path::Path,
    config: &util::execution_config::ExecutionConfig,
    db: &sea_orm::DatabaseConnection,
) -> Result<SubmissionDetailResponse, String> {
    let student_output_dir = submission_output_dir(
        assignment.module_id,
        assignment.id,
        submission.user_id,
        submission.attempt,
    );

    let mut student_outputs = Vec::new();

    //family reunion of if statments
    if let Ok(entries) = std::fs::read_dir(&student_output_dir) {
        for entry in entries.flatten() {
            let file_path = entry.path();
            if file_path.is_file() {
                if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                    if ext.eq_ignore_ascii_case("txt") {
                        if let Some(file_stem) = file_path.file_stem().and_then(|s| s.to_str()) {
                            if let Ok(output_id) = file_stem.parse::<i64>() {
                                if let Ok(Some(output)) =
                                    AssignmentSubmissionOutputModel::find_by_id(output_id)
                                        .one(db)
                                        .await
                                {
                                    if let Ok(Some(task)) =
                                        AssignmentTaskModel::find_by_id(output.task_id)
                                            .one(db)
                                            .await
                                    {
                                        if !task.code_coverage {
                                            student_outputs.push(file_path.clone());
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
    student_outputs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    let mut marking_job = MarkingJob::new(
        memo_outputs.to_vec(),
        student_outputs,
        mark_allocator_path.to_path_buf(),
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

    let mark_report = marking_job.mark().await.map_err(|e| format!("Marking Error: {:?}", e))?;

    let mark = MarkSummary {
        earned: mark_report.data.mark.earned,
        total: mark_report.data.mark.total,
    };
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
            let summary = cov.summary.as_ref().map(|s| MarkSummary { earned: s.earned, total: s.total });
            let files: Vec<serde_json::Value> = cov
                .files
                .iter()
                .map(|f| serde_json::json!({
                    "path": f.path,
                    "earned": f.earned,
                    "total": f.total,
                }))
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
        tasks,
        code_coverage,
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

/// Processes code execution for a submission
async fn process_submission_code(
    db: &sea_orm::DatabaseConnection,
    submission_id: i64,
    config: ExecutionConfig,
    module_id: i64,
    assignment_id: i64,
) -> Result<(), String> {
    if config.project.submission_mode == SubmissionMode::Manual.clone() {
        code_runner::create_submission_outputs_for_all_tasks(db, submission_id)
            .await
            .map_err(|e| format!("Code runner failed: {}", e))
    } else {
        ai::run_ga_job(db, submission_id, config, module_id, assignment_id)
            .await
            .map_err(|e| format!("GATLAM failed: {}", e))
    }
}

/// Clears the submission output directory
fn clear_submission_output(
    submission: &AssignmentSubmissionModel,
    module_id: i64,
    assignment_id: i64,
) -> Result<(), String> {
    let attempt = attempt_dir(module_id, assignment_id, submission.user_id, submission.attempt);
    let output_dir = attempt.join("submission_output");
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir)
            .map_err(|e| format!("Failed to clear output directory: {}", e))?;
    }

    let report_path = submission_report_path(module_id, assignment_id, submission.user_id, submission.attempt);
    if report_path.exists() {
        fs::remove_file(&report_path)
            .map_err(|e| format!("Failed to remove existing report: {}", e))?;
    }
    Ok(())
}

/// Read JSON into `SubmissionDetailResponse`, mutate, and write atomically.
async fn update_submission_report_marks(
    module_id: i64,
    assignment_id: i64,
    submission: &AssignmentSubmissionModel,
    new_mark: &MarkSummary,
    new_tasks: Option<&Vec<serde_json::Value>>,
) -> Result<(), String> {
    let report_path = submission_report_path(
        module_id,
        assignment_id,
        submission.user_id,
        submission.attempt,
    );

    let content = fs::read_to_string(&report_path)
        .map_err(|e| format!("Failed to read existing report: {}", e))?;

    let mut resp: SubmissionDetailResponse =
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to deserialize report: {}", e))?;

    resp.mark = MarkSummary { earned: new_mark.earned, total: new_mark.total };
    if let Some(tasks) = new_tasks {
        resp.tasks = tasks.clone();
    }
    resp.updated_at = Utc::now().to_rfc3339();

    let output = serde_json::to_string_pretty(&resp)
        .map_err(|e| format!("Failed to serialize updated report: {}", e))?;
    fs::write(&report_path, output)
        .map_err(|e| format!("Failed to write report: {}", e))?;

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
///     "tasks": [ ... ],
///     "code_coverage": [ ... ],
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
/// { "success": false, "message": "Failed to mark submission" }
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
pub async fn submit_assignment(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let db = app_state.db();

    let assignment = match load_assignment(module_id, assignment_id, db).await {
        Ok(assignment) => assignment,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<SubmissionDetailResponse>::error("Assignment not found")),
            );
        }
    };

    let mut is_practice: bool = false;
    let mut attests_ownership: bool = false;
    let mut file_name: Option<String> = None;
    let mut file_bytes: Option<bytes::Bytes> = None;

    // Parse multipart
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        match field.name() {
            Some("file") => {
                file_name = field.file_name().map(|s| s.to_string());
                file_bytes = Some(field.bytes().await.unwrap_or_default());
            }
            Some("is_practice") => {
                let val = field.text().await.unwrap_or_default();
                let v = val.trim().to_ascii_lowercase();
                is_practice = v == "true" || v == "1" || v == "on";
            }
            Some("attests_ownership") => {
                let val = field.text().await.unwrap_or_default();
                let v = val.trim().to_ascii_lowercase();
                attests_ownership = v == "true" || v == "1" || v == "on";
            }
            _ => {}
        }
    }

    // Require attestation (422)
    if !attests_ownership {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<SubmissionDetailResponse>::error(
                "You must confirm the ownership attestation before submitting",
            )),
        );
    }

    // Validate file presence, type, non-empty
    let (file_name, file_bytes) = match validate_file_upload(&file_name, &file_bytes) {
        Ok((name, bytes)) => (name, bytes),
        Err(response) => return response,
    };

    // Enforce max size (mirror UI 50MB)
    const MAX_SIZE_MB: usize = 50;
    if file_bytes.len() > MAX_SIZE_MB * 1024 * 1024 {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(ApiResponse::<SubmissionDetailResponse>::error(
                "File too large. Max size is 50 MB",
            )),
        );
    }

    // Policy check
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
                Json(ApiResponse::<SubmissionDetailResponse>::error(&msg)),
            );
        }
        Err(e) => {
            eprintln!("Attempt policy check failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<SubmissionDetailResponse>::error(
                    "Failed to evaluate submission attempts",
                )),
            );
        }
    }

    // Load execution config
    let config = match get_execution_config(module_id, assignment_id) {
        Ok(config) => config,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<SubmissionDetailResponse>::error(&e)),
            );
        }
    };

    // Scan for disallowed code (best-effort; failure just logs). Write to a temp file first.
    let disallowed_present = {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = match NamedTempFile::new() {
            Ok(f) => f,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<SubmissionDetailResponse>::error(&format!(
                        "Failed to create temp file: {}",
                        e
                    ))),
                );
            }
        };
        if let Err(e) = temp_file.write_all(&file_bytes) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<SubmissionDetailResponse>::error(&format!(
                    "Failed to write temp file: {}",
                    e
                ))),
            );
        }
        let temp_path = temp_file.into_temp_path();
        scan_disallowed_best_effort(&temp_path, &config)
    };

    let file_hash = format!("{:x}", md5::compute(&file_bytes));
    let attempt = match get_next_attempt(assignment_id, claims.sub, db).await {
        Ok(attempt) => attempt,
        Err(e) => {
            eprintln!("Error getting next attempt: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<SubmissionDetailResponse>::error(
                    "Failed to determine attempt number",
                )),
            );
        }
    };

    let submission = match AssignmentSubmissionModel::save_file(
        db,
        assignment_id,
        claims.sub,
        attempt,
        0,
        0,
        is_practice,
        &file_name,
        &file_hash,
        &file_bytes,
    )
    .await
    {
        Ok(model) => model,
        Err(e) => {
            eprintln!("Error saving submission: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<SubmissionDetailResponse>::error(
                    "Failed to save submission",
                )),
            );
        }
    };

    if let Err(e) =
        process_submission_code(db, submission.id, config.clone(), module_id, assignment_id).await
    {
        eprintln!("Code execution failed: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<SubmissionDetailResponse>::error(
                "Failed to run code for submission",
            )),
        );
    }

    if config.project.submission_mode != SubmissionMode::Manual {
        let tasks_res = assignment_task::Entity::find()
            .filter(assignment_task::Column::AssignmentId.eq(assignment_id))
            .all(db)
            .await;

        let tasks = match tasks_res {
            Ok(t) => t,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<SubmissionDetailResponse>::error(
                        "Failed to fetch assignment tasks",
                    )),
                );
            }
        };

        let memo_dir = memo_output_dir(module_id, assignment_id);
        let mut task_file_pairs = vec![];

        for task in &tasks {
            let task_info = TaskInfo {
                id: task.id,
                task_number: task.task_number,
                code_coverage: task.code_coverage,
                name: if task.name.trim().is_empty() {
                    format!("Task {}", task.task_number)
                } else {
                    task.name.clone()
                },
            };

            let memo_path = match assignment_memo_output::Entity::find()
                .filter(assignment_memo_output::Column::AssignmentId.eq(assignment_id))
                .filter(assignment_memo_output::Column::TaskId.eq(task.id))
                .one(db)
                .await
            {
                Ok(Some(m)) => storage_root().join(&m.path),
                Ok(None) => memo_dir.join(format!("no_memo_for_task_{}", task.id)),
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<SubmissionDetailResponse>::error(
                            "Failed to fetch memo output",
                        )),
                    );
                }
            };

            task_file_pairs.push((task_info, memo_path));
        }

        if let Err(_) = generate_allocator(module_id, assignment_id, &task_file_pairs).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<SubmissionDetailResponse>::error(
                    "Failed to generate allocator",
                )),
            );
        }
    }

    if let Err(e) = load_assignment_allocator(assignment.module_id, assignment.id).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<SubmissionDetailResponse>::error(&e)),
        );
    }

    let (_, mark_allocator_path, memo_outputs) =
        match get_assignment_paths(assignment.module_id, assignment.id) {
            Ok(paths) => paths,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<SubmissionDetailResponse>::error(&e)),
                );
            }
        };

    match grade_submission(
        submission.clone(),
        &assignment,
        &memo_outputs,
        &mark_allocator_path,
        &config,
        db,
    )
    .await
    {
        Ok(mut resp) => {
            if disallowed_present {
                if let Err(e) = enforce_zero_mark(
                    assignment.module_id,
                    assignment.id,
                    &submission,
                    resp.mark.total,
                    None,
                    db,
                )
                .await
                {
                    eprintln!("Failed to enforce zero mark: {}", e);
                }
                resp.mark = MarkSummary { earned: 0, total: resp.mark.total };

                return (
                    StatusCode::OK,
                    Json(ApiResponse::success(
                        resp,
                        "Submission received and graded (disallowed code detected: mark set to 0)",
                    )),
                );
            }

            (
                StatusCode::OK,
                Json(ApiResponse::success(resp, "Submission received and graded")),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<SubmissionDetailResponse>::error(e)),
        ),
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

    if let Err(e) = load_assignment_allocator(assignment.module_id, assignment.id).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<RemarkResponse>::error(e)),
        );
    }

    let (_, mark_allocator_path, memo_outputs) =
        match get_assignment_paths(assignment.module_id, assignment.id) {
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
            let mark_allocator_path = mark_allocator_path.clone();
            let config = config.clone();
            async move {
                let disallowed_present = scan_disallowed_best_effort(submission.full_path(), &config);
                let student_output_dir = submission_output_dir(
                    assignment.module_id,
                    assignment.id,
                    submission.user_id,
                    submission.attempt,
                );

                let mut student_outputs = Vec::new();
                if let Ok(entries) = std::fs::read_dir(&student_output_dir) {
                    for entry in entries.flatten() {
                        let file_path = entry.path();
                        if file_path.is_file() {
                            if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                                if ext.eq_ignore_ascii_case("txt") {
                                    student_outputs.push(file_path);
                                }
                            }
                        }
                    }
                }
                student_outputs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

                let mut marking_job = MarkingJob::new(
                    memo_outputs.to_vec(),
                    student_outputs,
                    mark_allocator_path.to_path_buf(),
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

                let mark_report = marking_job.mark().await.map_err(|e| format!("Marking Error: {:?}", e))?;

                let mark = MarkSummary {
                    earned: mark_report.data.mark.earned,
                    total: mark_report.data.mark.total,
                };

                let applied_mark = if disallowed_present {
                    if let Err(e) = enforce_zero_mark(
                        assignment.module_id,
                        assignment.id,
                        &submission,
                        mark.total,
                        None,
                        db,
                    )
                    .await
                    {
                        eprintln!("Failed to enforce zero mark (remark): {}", e);
                    }
                    MarkSummary { earned: 0, total: mark.total }
                } else {
                    mark
                };

                let tasks = serde_json::to_value(&mark_report.data.tasks)
                    .unwrap_or_default()
                    .as_array()
                    .cloned()
                    .unwrap_or_default();

                match update_submission_report_marks(
                    assignment.module_id,
                    assignment.id,
                    &submission,
                    &applied_mark,
                    Some(&tasks),
                ).await {
                    Ok(_) => Ok(()),
                    Err(_err) => {
                        let resp = grade_submission(
                            submission.clone(),
                            &assignment,
                            &memo_outputs,
                            &mark_allocator_path,
                            &config,
                            db,
                        )
                        .await?;

                        if disallowed_present {
                            if let Err(e) = enforce_zero_mark(
                                assignment.module_id,
                                assignment.id,
                                &submission,
                                resp.mark.total,
                                None,
                                db,
                            )
                            .await
                            {
                                eprintln!("Failed to enforce zero mark (remark fallback): {}", e);
                            }
                        }

                        Ok(())
                    },
                }
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

    if let Err(e) = load_assignment_allocator(assignment.module_id, assignment.id).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<ResubmitResponse>::error(e)),
        );
    }

    let (_, mark_allocator_path, memo_outputs) =
        match get_assignment_paths(assignment.module_id, assignment.id) {
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

    let (resubmitted, failed) =
        execute_bulk_operation(submission_ids.clone(), assignment_id, db, |submission| {
            let db = db.clone();
            let assignment = assignment.clone();
            let memo_outputs = memo_outputs.clone();
            let mark_allocator_path = mark_allocator_path.clone();
            let config = config.clone();
            async move {
                let disallowed_present = scan_disallowed_best_effort(submission.full_path(), &config);

                if let Err(e) = clear_submission_output(&submission, assignment.module_id, assignment.id) {
                    return Err(e);
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
                    return Err(format!("Failed to run code for submission: {}", e));
                }

                let resp = grade_submission(
                    submission.clone(),
                    &assignment,
                    &memo_outputs,
                    &mark_allocator_path,
                    &config,
                    &db,
                )
                .await?;

                if disallowed_present {
                    if let Err(e) = enforce_zero_mark(
                        assignment.module_id,
                        assignment.id,
                        &submission,
                        resp.mark.total,
                        None,
                        &db,
                    )
                    .await
                    {
                        eprintln!("Failed to enforce zero mark (resubmit): {}", e);
                    }
                }

                Ok(())
            }
        })
        .await;

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
