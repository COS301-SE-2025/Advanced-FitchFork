use super::common::{CodeComplexity, CodeComplexitySummary, MarkSummary, SubmissionDetailResponse};
use crate::{auth::AuthUser, response::ApiResponse, routes::modules::assignments::get::is_late};
use axum::{
    Json,
    extract::{Extension, Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use code_runner;
use db::models::{
    assignment::{Column as AssignmentColumn, Entity as AssignmentEntity},
    assignment_submission::{self, Model as AssignmentSubmissionModel},
};
use marker::MarkingJob;
use md5;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use util::{mark_allocator::mark_allocator::load_allocator, state::AppState};
use util::execution_config::ExecutionConfig;
use serde::{Serialize, Deserialize};
use tokio_util::bytes;

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
        (_, Some(true)) => {
            AssignmentSubmissionModel::find_by_assignment(assignment_id, db)
                .await
                .map_err(|e| format!("Failed to fetch submissions: {}", e))
        }
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
) -> Result<(std::path::PathBuf, std::path::PathBuf, Vec<std::path::PathBuf>), String> {
    let assignment_storage_root = std::env::var("ASSIGNMENT_STORAGE_ROOT")
        .unwrap_or_else(|_| "data/assignment_files".to_string());
    
    let base_path = std::path::PathBuf::from(&assignment_storage_root)
        .join(format!("module_{}", module_id))
        .join(format!("assignment_{}", assignment_id));
    
    let mark_allocator_path = base_path.join("mark_allocator/allocator.json");
    let memo_output_dir = base_path.join("memo_output");

    let mut memo_outputs: Vec<_> = match std::fs::read_dir(&memo_output_dir) {
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
    base_path: &std::path::Path,
    memo_outputs: &[std::path::PathBuf],
    mark_allocator_path: &std::path::Path,
    config: &util::execution_config::ExecutionConfig,
) -> Result<SubmissionDetailResponse, String> {
    let student_output_dir = base_path
        .join("assignment_submissions")
        .join(format!("user_{}", submission.user_id))
        .join(format!("attempt_{}", submission.attempt))
        .join("submission_output");

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

    let marking_job = MarkingJob::new(
        memo_outputs.to_vec(),
        student_outputs,
        mark_allocator_path.to_path_buf(),
        config.clone(),
    );

    let mark_report = marking_job.mark().await.map_err(|e| {
        eprintln!("MARKING ERROR: {:#?}", e);
        format!("{:?}", e)
    })?;

    let mark = MarkSummary {
        earned: mark_report.data.mark.earned,
        total: mark_report.data.mark.total,
    };
    let tasks = serde_json::to_value(&mark_report.data.tasks)
        .unwrap_or_default()
        .as_array()
        .cloned()
        .unwrap_or_default();
    let code_coverage = match &mark_report.data.code_coverage {
        Some(cov) => {
            let arr = serde_json::to_value(cov)
                .unwrap_or_default()
                .as_array()
                .cloned()
                .unwrap_or_default();
            if !arr.is_empty() { Some(arr) } else { None }
        }
        None => None,
    };
    let code_complexity = match &mark_report.data.code_complexity {
        Some(c) => {
            let metrics = serde_json::to_value(&c.metrics)
                .unwrap_or_default()
                .as_array()
                .cloned()
                .unwrap_or_default();
            let summary = CodeComplexitySummary {
                earned: c.summary.as_ref().map(|s| s.earned).unwrap_or(0),
                total: c.summary.as_ref().map(|s| s.total).unwrap_or(0),
            };
            if !metrics.is_empty() || summary.earned != 0 || summary.total != 0 {
                Some(CodeComplexity { summary, metrics })
            } else {
                None
            }
        }
        None => None,
    };

    let resp = SubmissionDetailResponse {
        id: submission.id,
        attempt: submission.attempt,
        filename: submission.filename.clone(),
        hash: submission.file_hash.clone(),
        created_at: submission.created_at.to_rfc3339(),
        updated_at: submission.updated_at.to_rfc3339(),
        mark,
        is_practice: submission.is_practice,
        is_late: is_late(submission.created_at, assignment.due_date),
        tasks,
        code_coverage,
        code_complexity,
    };

    let attempt_dir = base_path
        .join("assignment_submissions")
        .join(format!("user_{}", submission.user_id))
        .join(format!("attempt_{}", submission.attempt));
    let report_path = attempt_dir.join("submission_report.json");
    if let Ok(json) = serde_json::to_string_pretty(&resp) {
        std::fs::write(&report_path, json).map_err(|e| e.to_string())?;
    } else {
        return Err("Failed to serialize submission report".to_string());
    }

    Ok(resp)
}

/// Processes code execution for a submission
async fn process_submission_code(
    db: &sea_orm::DatabaseConnection,
    submission_id: i64,
) -> Result<(), String> {
    code_runner::create_submission_outputs_for_all_tasks(db, submission_id)
        .await
        .map_err(|e| format!("Code runner failed: {}", e))
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
/// code execution, mark allocation, and optional code coverage/complexity analysis.
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
///     "code_complexity": {
///       "summary": { "earned": 10, "total": 15 },
///       "metrics": [ ... ]
///     }
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
/// ### Filesystem
/// - Uploaded file and outputs are stored under:
///   `ASSIGNMENT_STORAGE_ROOT/module_{module_id}/assignment_{assignment_id}/assignment_submissions/user_{user_id}/attempt_{n}/`
///
/// ### Notes
/// - Each submission increments the attempt number for the user/assignment
/// - Only one file per submission is accepted
/// - Practice submissions are marked and reported but may not count toward final grade
/// - The returned report includes detailed per-task grading, code coverage, and complexity if available
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
                Json(ApiResponse::<SubmissionDetailResponse>::error(
                    "Assignment not found",
                )),
            );
        }
    };

    let mut is_practice: bool = false;
    let mut file_name: Option<String> = None;
    let mut file_bytes: Option<bytes::Bytes> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        match field.name() {
            Some("file") => {
                file_name = field.file_name().map(|s| s.to_string());
                file_bytes = Some(field.bytes().await.unwrap_or_default());
            }
            Some("is_practice") => {
                let val = field.text().await.unwrap_or_default();
                is_practice = val == "true" || val == "1";
            }
            _ => {}
        }
    }

    let (file_name, file_bytes) = match validate_file_upload(&file_name, &file_bytes) {
        Ok((name, bytes)) => (name, bytes),
        Err(response) => return response,
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

    if let Err(e) = process_submission_code(db, submission.id).await {
        eprintln!("Code execution failed: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<SubmissionDetailResponse>::error(
                "Failed to run code for submission",
            )),
        );
    }

    if let Err(e) = load_assignment_allocator(assignment.module_id, assignment.id).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<SubmissionDetailResponse>::error(&e)),
        );
    }

    let (base_path, mark_allocator_path, memo_outputs) = match get_assignment_paths(assignment.module_id, assignment.id) {
        Ok(paths) => paths,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<SubmissionDetailResponse>::error(&e)),
            );
        }
    };

    let config = match get_execution_config(module_id, assignment_id) {
        Ok(config) => config,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<SubmissionDetailResponse>::error(&e)),
            );
        }
    };

    match grade_submission(
        submission,
        &assignment,
        &base_path,
        &memo_outputs,
        &mark_allocator_path,
        &config,
    )
    .await
    {
        Ok(resp) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                resp,
                "Submission received and graded",
            )),
        ),
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

    let submission_ids = match resolve_submission_ids(req.submission_ids, req.all, assignment_id, db).await {
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

    let (base_path, mark_allocator_path, memo_outputs) = match get_assignment_paths(assignment.module_id, assignment.id) {
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

    let (regraded, failed) = execute_bulk_operation(
        submission_ids.clone(),
        assignment_id,
        db,
        |submission| {
            let assignment = assignment.clone();
            let base_path = base_path.clone();
            let memo_outputs = memo_outputs.clone();
            let mark_allocator_path = mark_allocator_path.clone();
            let config = config.clone();
            async move {
                grade_submission(
                    submission,
                    &assignment,
                    &base_path,
                    &memo_outputs,
                    &mark_allocator_path,
                    &config,
                )
                .await
                .map(|_| ())
            }
        },
    ).await;

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
                Json(ApiResponse::<ResubmitResponse>::error("Assignment not found")),
            );
        }
    };

    let submission_ids = match resolve_submission_ids(req.submission_ids, req.all, assignment_id, db).await {
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

    let (base_path, mark_allocator_path, memo_outputs) = match get_assignment_paths(assignment.module_id, assignment.id) {
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

    let (resubmitted, failed) = execute_bulk_operation(
        submission_ids.clone(),
        assignment_id,
        db,
        |submission| {
            let db = db.clone();
            let assignment = assignment.clone();
            let base_path = base_path.clone();
            let memo_outputs = memo_outputs.clone();
            let mark_allocator_path = mark_allocator_path.clone();
            let config = config.clone();
            async move {
                if let Err(e) = process_submission_code(&db, submission.id).await {
                    return Err(format!("Failed to run code for submission: {}", e));
                }

                grade_submission(
                    submission,
                    &assignment,
                    &base_path,
                    &memo_outputs,
                    &mark_allocator_path,
                    &config,
                )
                .await
                .map(|_| ())
            }
        },
    ).await;

    let response = ResubmitResponse { resubmitted, failed };
    let message = format!("Resubmitted {}/{} submissions", resubmitted, submission_ids.len());

    (
        StatusCode::OK,
        Json(ApiResponse::success(response, &message)),
    )
}