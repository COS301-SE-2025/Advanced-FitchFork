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

// Common grading function that can be used for both initial submissions and regrading
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
            let entry_path = entry.path();
            if entry_path.is_dir() {
                if let Some(dir_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                    if dir_name.starts_with("task_") {
                        if let Ok(task_entries) = std::fs::read_dir(&entry_path) {
                            for task_entry in task_entries.flatten() {
                                let file_path = task_entry.path();
                                if file_path.is_file() {
                                    if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                                        if ext.eq_ignore_ascii_case("txt") {
                                            student_outputs.push(file_path);
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

    let marking_job = MarkingJob::new(
        memo_outputs.to_vec(),
        student_outputs,
        mark_allocator_path.to_path_buf(),
        config.clone(),
    );
    let mark_report = marking_job.mark().await.map_err(|e| format!("{:?}", e))?;

    let mark = MarkSummary {
        earned: mark_report.data.mark.earned as i64,
        total: mark_report.data.mark.total as i64,
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
                earned: c.summary.as_ref().map(|s| s.earned as i64).unwrap_or(0),
                total: c.summary.as_ref().map(|s| s.total as i64).unwrap_or(0),
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
        id: submission.id as i64,
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

    let assignment = AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id as i32))
        .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
        .one(db)
        .await
        .unwrap()
        .unwrap();

    let mut is_practice: bool = false;
    let mut file_name: Option<String> = None;
    let mut file_bytes: Option<_> = None;

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

    let file_name = match file_name {
        Some(name) => name,
        None => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiResponse::<SubmissionDetailResponse>::error(
                    "No file provided",
                )),
            );
        }
    };
    let file_bytes = match file_bytes {
        Some(bytes) => bytes,
        None => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiResponse::<SubmissionDetailResponse>::error(
                    "No file provided",
                )),
            );
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
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<SubmissionDetailResponse>::error(
                "Only .tgz, .gz, .tar, and .zip files are allowed",
            )),
        );
    }
    if file_bytes.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<SubmissionDetailResponse>::error(
                "Empty file provided",
            )),
        );
    }

    let file_hash = format!("{:x}", md5::compute(&file_bytes));

    // TODO: prevent race conditions here with a lock or something from COS226
    let prev_attempt = assignment_submission::Entity::find()
        .filter(assignment_submission::Column::AssignmentId.eq(assignment_id as i32))
        .filter(assignment_submission::Column::UserId.eq(claims.sub))
        .order_by_desc(assignment_submission::Column::Attempt)
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|s| s.attempt)
        .unwrap_or(0);
    let attempt = prev_attempt + 1;

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

    if let Err(e) = code_runner::create_submission_outputs_for_all_tasks(&db, submission.id).await {
        eprintln!("Code runner failed: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<SubmissionDetailResponse>::error(
                "Failed to run code for submission",
            )),
        );
    }

    match load_allocator(assignment.module_id, assignment.id).await {
        Ok(_) => {}
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<SubmissionDetailResponse>::error(
                    "Failed to load mark allocator",
                )),
            );
        }
    };

    let assignment_storage_root = std::env::var("ASSIGNMENT_STORAGE_ROOT")
        .unwrap_or_else(|_| "data/assignment_files".to_string());
    let base_path = std::path::PathBuf::from(&assignment_storage_root)
        .join(format!("module_{}", assignment.module_id))
        .join(format!("assignment_{}", assignment.id));
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

    let config = ExecutionConfig::get_execution_config(module_id, assignment_id)
        .expect("Failed to load execution config");

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

#[derive(Debug, Deserialize)]
pub struct RemarkRequest {
    #[serde(default)]
    submission_ids: Option<Vec<i64>>,
    #[serde(default)]
    all: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct RemarkResponse {
    regraded: usize,
    failed: Vec<FailedRemark>,
}

#[derive(Debug, Serialize)]
pub struct FailedRemark {
    id: Option<i64>,
    error: String,
}

// TODO: Allow ALs to use this endpoint. At the time of implementing this, ALs were not implemented.
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

    let submission_ids = match (req.submission_ids, req.all) {
        (Some(ids), _) if !ids.is_empty() => ids,
        (_, Some(true)) => {
            match AssignmentSubmissionModel::find_by_assignment(assignment_id, db).await {
                Ok(ids) => ids,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<RemarkResponse>::error(format!(
                            "Failed to fetch submissions: {}",
                            e
                        ))),
                    )
                }
            }
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<RemarkResponse>::error(
                    "Must provide either submission_ids or all=true",
                )),
            )
        }
    };

    let assignment = AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id as i32))
        .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
        .one(db)
        .await
        .unwrap()
        .unwrap();

    match load_allocator(assignment.module_id, assignment.id).await {
        Ok(_) => {}
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<RemarkResponse>::error(
                    "Failed to load mark allocator",
                )),
            );
        }
    };

    let assignment_storage_root = std::env::var("ASSIGNMENT_STORAGE_ROOT")
        .unwrap_or_else(|_| "data/assignment_files".to_string());
    let base_path = std::path::PathBuf::from(&assignment_storage_root)
        .join(format!("module_{}", assignment.module_id))
        .join(format!("assignment_{}", assignment.id));
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

    let config = ExecutionConfig::get_execution_config(module_id, assignment_id)
        .expect("Failed to load execution config");

    let mut regraded = 0;
    let mut failed = Vec::new();

    for submission_id in submission_ids.clone() {
        let submission = match assignment_submission::Entity::find_by_id(submission_id as i32)
            .one(db)
            .await
        {
            Ok(Some(sub)) => sub,
            Ok(None) => {
                failed.push(FailedRemark {
                    id: Some(submission_id),
                    error: "Submission not found".to_string(),
                });
                continue;
            }
            Err(e) => {
                failed.push(FailedRemark {
                    id: Some(submission_id),
                    error: format!("Database error: {}", e),
                });
                continue;
            }
        };

        if submission.assignment_id != assignment_id {
            failed.push(FailedRemark {
                id: Some(submission_id),
                error: "Submission does not belong to this assignment".to_string(),
            });
            continue;
        }

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
            Ok(_) => regraded += 1,
            Err(e) => failed.push(FailedRemark {
                id: Some(submission_id),
                error: e,
            }),
        }
    }

    let response = RemarkResponse { regraded, failed };
    let message = format!("Regraded {}/{} submissions", regraded, submission_ids.len());

    (
        StatusCode::OK,
        Json(ApiResponse::success(response, &message)),
    )
}