//! # Assignment Submission Endpoint
//!
//! This module implements the POST endpoint for submitting assignment solutions in FitchFork.
//! It handles file uploads, validates and stores submissions, triggers code execution and marking,
//! and returns a detailed grading report. The same report is also saved as a JSON file for auditing.
//!
//! ## Endpoint
//! - **POST** `/modules/:module_id/assignments/:assignment_id/submissions`
//!
//! ## Request
//! - Multipart form data with fields:
//!   - `file`: The assignment submission archive (.tgz, .gz, .tar, .zip)
//!   - `is_practice`: Optional, whether this is a practice submission
//!
//! ## Response
//! - On success: JSON with detailed grading report (see [`SubmissionDetailResponse`])
//! - On error: JSON with error message
//!
//! ## Logic Flow
//! 1. Validate assignment and user
//! 2. Parse and validate uploaded file
//! 3. Store the file and create a DB record
//! 4. Run code runner to generate outputs
//! 5. Load marking schema (allocator)
//! 6. Collect memo and student outputs
//! 7. Run marking logic (see marker crate)
//! 8. Build and return a detailed grading report
//! 9. Save the same report as `submission_report.json` in the attempt folder
//!
//! ## Filesystem Layout
//! - `assignment_submissions/user_{id}/attempt_{n}/submission_output/` (per-task outputs)
//! - `assignment_submissions/user_{id}/attempt_{n}/submission_report.json` (grading report)
//! - `assignment_submissions/user_{id}/attempt_{n}/...zip` (uploaded file)

use axum::{extract::{Path, Multipart, Extension}, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use crate::auth::AuthUser;
use crate::response::ApiResponse;
use db::connect;
use db::models::assignment_submission::Model as AssignmentSubmissionModel;
use db::models::assignment_submission;
use db::models::assignment::{Entity as AssignmentEntity, Column as AssignmentColumn};
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, QueryOrder};
use code_runner;
use util::mark_allocator::mark_allocator::load_allocator;
use marker::MarkingJob;
use crate::routes::modules::assignments::get::is_late;
use md5;

/// Summary of marks for a submission.
#[derive(Debug, Serialize)]
pub struct MarkSummary {
    pub earned: i64,
    pub total: i64,
}

/// Summary of code complexity metrics.
#[derive(Debug, Serialize)]
pub struct CodeComplexitySummary {
    pub earned: i64,
    pub total: i64,
}

/// Code complexity details for a submission.
#[derive(Debug, Serialize)]
pub struct CodeComplexity {
    pub summary: CodeComplexitySummary,
    pub metrics: Vec<serde_json::Value>, // Placeholder, adjust as needed
}

/// The full response returned after a submission is processed and graded.
///
/// Fields:
/// - `id`: Submission DB ID
/// - `attempt`: Attempt number for this user/assignment
/// - `filename`: Name of the uploaded file
/// - `hash`: MD5 hash of the uploaded file
/// - `created_at`, `updated_at`: Timestamps
/// - `mark`: Earned/total marks
/// - `is_practice`: Whether this was a practice submission
/// - `is_late`: Whether the submission was after the due date
/// - `tasks`: Per-task grading details
/// - `code_coverage`: Optional code coverage report (if available)
/// - `code_complexity`: Optional code complexity report (if available)
#[derive(Debug, Serialize)]
pub struct SubmissionDetailResponse {
    pub id: i64,
    pub attempt: i64,
    pub filename: String,
    pub hash: String,
    pub created_at: String,
    pub updated_at: String,
    pub mark: MarkSummary,
    pub is_practice: bool,
    pub is_late: bool,
    pub tasks: Vec<serde_json::Value>, // Placeholder, adjust as needed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_coverage: Option<Vec<serde_json::Value>>, // Placeholder, adjust as needed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_complexity: Option<CodeComplexity>,
}

/// Handles assignment submission uploads, grading, and report generation.
///
/// # Arguments
/// - `Path((module_id, assignment_id))`: Module and assignment IDs from the URL
/// - `Extension(AuthUser(claims))`: Authenticated user context
/// - `multipart`: Multipart form data (file upload)
///
/// # Returns
/// - On success: `(StatusCode::OK, Json(ApiResponse<SubmissionDetailResponse>))`
/// - On error: Appropriate error status and message
///
/// # Side Effects
/// - Saves the uploaded file and generated outputs to disk
/// - Triggers code execution and marking
/// - Saves a copy of the grading report as `submission_report.json` in the attempt folder
///
/// # Filesystem
/// - Uploaded file and outputs are stored under:
///   `data/assignment_files/module_{module_id}/assignment_{assignment_id}/assignment_submissions/user_{user_id}/attempt_{n}/`
///
/// # Errors
/// - Returns 404 if assignment not found
/// - Returns 422 if file is missing or invalid
/// - Returns 500 for internal errors (DB, marking, etc)
#[allow(clippy::too_many_lines)]
pub async fn submit_assignment(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let db = connect().await;

    let assignment = match AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id as i32))
        .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
        .one(&db)
        .await
    {
        Ok(Some(a)) => a,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<SubmissionDetailResponse> ::error("Assignment not found")),
            )
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<SubmissionDetailResponse> ::error("Database error")),
            )
        }
    };

    let mut is_practice = false;
    // let mut force_submit = false;
    let mut file_name = None;
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
            // Some("force_submit") => {
            //     let val = field.text().await.unwrap_or_default();
            //     force_submit = val == "true" || val == "1";
            // }
            _ => {}
        }
    }

    let file_name = match file_name {
        Some(name) => name,
        None => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiResponse::<SubmissionDetailResponse> ::error("No file provided")),
            )
        }
    };
    let file_bytes = match file_bytes {
        Some(bytes) => bytes,
        None => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiResponse::<SubmissionDetailResponse> ::error("No file provided")),
            )
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
            Json(ApiResponse::<SubmissionDetailResponse> ::error(
                "Only .tgz, .gz, .tar, and .zip files are allowed",
            )),
        )
    }
    if file_bytes.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<SubmissionDetailResponse> ::error("Empty file provided")),
        )
    }

    let file_hash = format!("{:x}", md5::compute(&file_bytes));

    let prev_attempt = assignment_submission::Entity::find()
        .filter(assignment_submission::Column::AssignmentId.eq(assignment_id as i32))
        .filter(assignment_submission::Column::UserId.eq(claims.sub))
        .order_by_desc(assignment_submission::Column::Attempt)
        .one(&db)
        .await
        .ok()
        .flatten()
        .map(|s| s.attempt)
        .unwrap_or(0);
    let attempt = prev_attempt + 1;

    let saved = match AssignmentSubmissionModel::save_file(
        &db,
        assignment_id,
        claims.sub,
        attempt,
        &file_name,
        &file_bytes,
    )
    .await
    {
        Ok(model) => model,
        Err(e) => {
            eprintln!("Error saving submission: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<SubmissionDetailResponse> ::error("Failed to save submission")),
            )
        }
    };

    if let Err(e) = code_runner::create_submission_outputs_for_all_tasks(&db, saved.id).await {
        eprintln!("Code runner failed: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<SubmissionDetailResponse> ::error("Failed to run code for submission")),
        )
    }

    let _allocator_json = match load_allocator(module_id, assignment_id).await {
        Ok(val) => val,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<SubmissionDetailResponse> ::error("Failed to load mark allocator")),
            )
        }
    };

    let assignment_storage_root = std::env::var("ASSIGNMENT_STORAGE_ROOT").unwrap_or_else(|_| "data/assignment_files".to_string());
    let base_path = std::path::PathBuf::from(&assignment_storage_root)
        .join(format!("module_{}", module_id))
        .join(format!("assignment_{}", assignment_id));
    let mark_allocator_path = base_path.join("mark_allocator/allocator.json");
    let memo_output_dir = base_path.join("memo_output");
    let student_output_dir = base_path
        .join("assignment_submissions")
        .join(format!("user_{}", claims.sub))
        .join(format!("attempt_{}", saved.attempt))
        .join("submission_output");

    let mut student_outputs = Vec::new();
    if let Ok(task_dirs) = std::fs::read_dir(&student_output_dir) {
        for task_dir in task_dirs.flatten() {
            let path = task_dir.path();
            if path.is_dir() {
                if let Ok(files) = std::fs::read_dir(&path) {
                    for file in files.flatten() {
                        let file_path = file.path();
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
    let memo_outputs: Vec<_> = match std::fs::read_dir(&memo_output_dir) {
        Ok(rd) => rd.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.is_file()).collect(),
        Err(_) => Vec::new(),
    };

    let marking_job = MarkingJob::new(
        memo_outputs,
        student_outputs,
        mark_allocator_path,
    );
    let mark_report = match marking_job.mark().await {
        Ok(report) => report,
        Err(e) => {
            eprintln!("Marking failed: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<SubmissionDetailResponse> ::error("Failed to mark submission")),
            )
        }
    };

    let mark = MarkSummary {
        earned: mark_report.data.mark.earned as i64,
        total: mark_report.data.mark.total as i64,
    };
    let tasks = serde_json::to_value(&mark_report.data.tasks).unwrap_or_default().as_array().cloned().unwrap_or_default();
    let code_coverage = match &mark_report.data.code_coverage {
        Some(cov) => {
            let arr = serde_json::to_value(cov).unwrap_or_default().as_array().cloned().unwrap_or_default();
            if !arr.is_empty() { Some(arr) } else { None }
        },
        None => None,
    };
    let code_complexity = match &mark_report.data.code_complexity {
        Some(c) => {
            let metrics = serde_json::to_value(&c.metrics).unwrap_or_default().as_array().cloned().unwrap_or_default();
            let summary = CodeComplexitySummary {
                earned: c.summary.as_ref().map(|s| s.earned as i64).unwrap_or(0),
                total: c.summary.as_ref().map(|s| s.total as i64).unwrap_or(0),
            };
            if !metrics.is_empty() || summary.earned != 0 || summary.total != 0 {
                Some(CodeComplexity { summary, metrics })
            } else {
                None
            }
        },
        None => None,
    };
    let resp = SubmissionDetailResponse {
        id: saved.id,
        attempt: saved.attempt,
        filename: saved.filename,
        hash: file_hash,
        created_at: saved.created_at.to_rfc3339(),
        updated_at: saved.updated_at.to_rfc3339(),
        mark,
        is_practice,
        is_late: is_late(saved.created_at, assignment.due_date),
        tasks,
        code_coverage,
        code_complexity,
    };

    let attempt_dir = base_path
        .join("assignment_submissions")
        .join(format!("user_{}", claims.sub))
        .join(format!("attempt_{}", saved.attempt));
    let report_path = attempt_dir.join("submission_report.json");
    if let Ok(json) = serde_json::to_string_pretty(&resp) {
        if let Err(e) = std::fs::write(&report_path, json) {
            eprintln!("Failed to write submission_report.json: {}", e);
        }
    } else {
        eprintln!("Failed to serialize submission report to JSON");
    }

    (StatusCode::OK, Json(ApiResponse::<SubmissionDetailResponse>::success(resp, "Submission received and graded")))
}