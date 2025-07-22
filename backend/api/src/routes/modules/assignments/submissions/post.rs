use axum::{
    extract::{Path, Multipart, Extension},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use db::{
    connect,
    models::{
        assignment_submission::{self, Model as AssignmentSubmissionModel},
        assignment::{Entity as AssignmentEntity, Column as AssignmentColumn},
    },
};
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, QueryOrder};
use crate::{
    auth::AuthUser,
    response::ApiResponse,
    routes::modules::assignments::get::is_late,
};
use code_runner;
use util::mark_allocator::mark_allocator::load_allocator;
use marker::MarkingJob;
use md5;
use super::common::{
    MarkSummary, CodeComplexitySummary, CodeComplexity, SubmissionDetailResponse,
};

/// POST /api/modules/:module_id/assignments/:assignment_id/submissions
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
    student_outputs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    let mut memo_outputs: Vec<_> = match std::fs::read_dir(&memo_output_dir) {
        Ok(rd) => rd.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.is_file()).collect(),
        Err(_) => Vec::new(),
    };
    memo_outputs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

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

/*
Testing plan

Happy Path (Success) Test Cases
1. Valid submission (zip file): Upload a valid .zip file as a student assigned to the module. Expect 200 OK, grading report returned.
2. Valid submission (tar file): Upload a valid .tar file as a student assigned to the module. Expect 200 OK, grading report returned.
3. Valid submission (tgz file): Upload a valid .tgz file as a student assigned to the module. Expect 200 OK, grading report returned.
4. Valid submission (gz file): Upload a valid .gz file as a student assigned to the module. Expect 200 OK, grading report returned.
5. Practice submission: Upload a valid file with is_practice=true. Expect 200 OK, is_practice field is true in response.
6. Multiple attempts: Submit twice as the same student; verify attempt increments. Expect 200 OK, attempt increases by 1.
7. Submission with code coverage and complexity: Upload a file that triggers code coverage and complexity analysis. Expect 200 OK, code_coverage and code_complexity fields present.
8. Submission exactly at due date: Submit a file with a timestamp exactly matching the assignment due date. Expect 200 OK, is_late is false.
9. Submission just after due date: Submit a file just after the due date. Expect 200 OK, is_late is true.
10. Submission with large file (within allowed size): Upload a large but valid file. Expect 200 OK, grading report returned.

Unhappy Path (Error) Test Cases
11. Missing file: Submit without a file in the form. Expect 422 Unprocessable Entity, error message about missing file.
12. Empty file: Submit an empty file. Expect 422 Unprocessable Entity, error message about empty file.
13. Invalid file extension: Submit a file with an unsupported extension (e.g., .exe). Expect 422 Unprocessable Entity, error message about allowed extensions.
14. Corrupted archive: Submit a corrupted .zip file. Expect 500 Internal Server Error, error message about failed extraction or grading.
15. Assignment not found: Submit to a non-existent assignment ID. Expect 404 Not Found, error message about assignment not found.
16. User not assigned to module: Submit as a user not assigned to the module. Expect 403 Forbidden or 404 Not Found, error message about permissions.
17. Database error during save: Simulate a database failure when saving the submission. Expect 500 Internal Server Error, error message about failed to save submission.
18. Failure in code runner: Simulate a failure in the code runner after file upload. Expect 500 Internal Server Error, error message about failed to run code.
19. Failure to load mark allocator: Simulate missing or invalid mark allocator. Expect 500 Internal Server Error, error message about failed to load mark allocator.
20. Failure in marking: Simulate a marking error (e.g., invalid memo output). Expect 500 Internal Server Error, error message about failed to mark submission.
*/