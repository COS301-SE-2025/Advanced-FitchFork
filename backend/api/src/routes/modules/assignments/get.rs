//! Assignment routes and response models.
//!
//! Provides endpoints and data structures for managing assignments within modules:
//!
//! - `GET /api/modules/{module_id}/assignments/{assignment_id}`  
//!   Retrieve a specific assignment along with its associated files.
//!
//! - `GET /api/modules/{module_id}/assignments`  
//!   Retrieve a paginated and optionally filtered list of assignments.
//!
//! - `GET /api/modules/{module_id}/assignments/{assignment_id}/stats`  
//!   Retrieve submission statistics for a specific assignment.
//!
//! - `GET /api/modules/{module_id}/assignments/{assignment_id}/readiness`  
//!   Retrieve a readiness report for a specific assignment, checking whether all required components are present.
//!
//! **Models:**  
//! - `AssignmentFileResponse`: Assignment data plus associated files.  
//! - `FilterReq` / `FilterResponse`: Query and response for paginated assignment lists.  
//! - `StatResponse` / `PerStudentSubmission`: Assignment submission statistics.  
//! - `AssignmentReadiness`: Detailed readiness report for an assignment.
//!
//! All endpoints use `AppState` for database access and return JSON-wrapped `ApiResponse`.

use crate::routes::modules::assignments::common::{AssignmentResponse, File};
use crate::{auth::AuthUser, response::ApiResponse};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, sea_query::Expr,
};
use serde::{Deserialize, Serialize};
use services::assignment::{Assignment, AssignmentService};
use services::assignment_file::AssignmentFileService;
use services::assignment_submission::AssignmentSubmissionService;
use services::service::Service;
use services::user::UserService;
use services::user_module_role::UserModuleRoleService;
use std::collections::HashMap;
use util::filters::{FilterParam, QueryParam};
use util::{
    execution_config::{
        ExecutionConfig,
        execution_config::{GradingPolicy, SubmissionMode},
    },
    state::AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssignmentFileResponse {
    pub assignment: AssignmentResponse,
    pub files: Vec<File>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_mark: Option<BestMark>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attempts: Option<AttemptsInfo>,
    // NEW: policy hints that are safe to show to students
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<AssignmentPolicy>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BestMark {
    pub earned: i64,
    pub total: i64,
    pub attempt: i64,
    pub submission_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttemptsInfo {
    pub used: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining: Option<u32>,
    pub can_submit: bool,
    pub limit_attempts: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssignmentPolicy {
    pub allow_practice_submissions: bool,
    pub submission_mode: SubmissionMode,
    pub grading_policy: GradingPolicy,
    pub limit_attempts: bool,
    pub pass_mark: u32,
}

impl From<AssignmentModel> for AssignmentFileResponse {
    fn from(assignment: AssignmentModel) -> Self {
        Self {
            assignment: AssignmentResponse {
                id: assignment.id,
                module_id: assignment.module_id as i64,
                name: assignment.name,
                description: assignment.description,
                status: assignment.status.to_string(),
                assignment_type: assignment.assignment_type.to_string(),
                available_from: assignment.available_from.to_rfc3339(),
                due_date: assignment.due_date.to_rfc3339(),
                created_at: assignment.created_at.to_rfc3339(),
                updated_at: assignment.updated_at.to_rfc3339(),
            },
            files: Vec::new(),
            best_mark: None,
            attempts: None,
            policy: None,
        }
    }
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}
///
/// Retrieve a specific assignment along with its associated files.  
/// Accessible to all users assigned to the module.
///
/// The response also includes a **student-safe policy summary** (`policy`) with
/// the key settings needed by the UI (e.g., practice allowed, attempt limits,
/// pass mark, submission mode, grading policy). Students cannot fetch the full
/// raw config; this `policy` is the safe subset exposed to all callers.
///
/// If the authenticated user is a **student** in this module, the response will
/// additionally include:
/// - `best_mark`: Their current best/last mark according to the assignment's grading policy
///   (practice and ignored submissions are excluded from this calculation)
/// - `attempts`: Attempt usage and remaining attempts derived from the assignment's policy
///   (only **non-practice** and **non-ignored** submissions count toward `used`)
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to retrieve
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "message": "Assignment retrieved successfully",
///   "data": {
///     "assignment": {
///       "id": 123,
///       "module_id": 456,
///       "name": "Assignment 1",
///       "description": "This is a sample assignment",
///       "assignment_type": "assignment",
///       "status": "open",
///       "available_from": "2024-01-01T00:00:00Z",
///       "due_date": "2024-01-31T23:59:59Z",
///       "created_at": "2024-01-01T00:00:00Z",
///       "updated_at": "2024-01-15T12:00:00Z"
///     },
///     "files": [
///       {
///         "id": "789",
///         "filename": "assignment.pdf",
///         "path": "module_456/assignment_123/assignment.pdf",
///         "file_type": "config",
///         "created_at": "2024-01-01T00:00:00Z",
///         "updated_at": "2024-01-01T00:00:00Z"
///       }
///     ],
///     "policy": {
///       "allow_practice_submissions": true,
///       "submission_mode": "manual",
///       "grading_policy": "last",
///       "limit_attempts": true,
///       "pass_mark": 50
///     },
///     "best_mark": {
///       "earned": 85,
///       "total": 100,
///       "attempt": 2,
///       "submission_id": 4567
///     },
///     "attempts": {
///       "used": 2,
///       "max": 3,
///       "remaining": 1,
///       "can_submit": true,
///       "limit_attempts": true
///     }
///   }
/// }
/// ```
///
/// > Notes:
/// > - `policy` is included for all callers and exposes only safe fields needed by the UI.
/// > - `best_mark` and `attempts` are **omitted** when the caller is not a student in this module.
/// > - `attempts.used` counts only **non-practice** and **non-ignored** submissions.
/// > - `attempts.can_submit` refers to whether the user may submit another **non-practice** attempt
/// >   (students obey `limit_attempts`; staff are always allowed).
/// > - `attempts.max` and `policy.max_attempts` are `null` when `limit_attempts` is `false`
/// >   (i.e., effectively unlimited).
///
/// - `404 Not Found`  
/// Assignment does not exist in this module.
/// ```json
/// { "success": false, "message": "Assignment not found" }
/// ```
///
/// - `500 Internal Server Error`  
/// Returned if the database or file fetch fails.
/// ```json
/// { "success": false, "message": "Failed to retrieve files: <error details>" }
/// ```
pub async fn get_assignment(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    let db = app_state.db();

    let assignment_res = assignment::Entity::find()
        .filter(assignment::Column::Id.eq(assignment_id as i32))
        .filter(assignment::Column::ModuleId.eq(module_id as i32))
        .one(db)
        .await;

    match assignment_res {
        Ok(Some(mut a)) => {
            // --- auto-adjust status (adjacent Ready↔Open↔Closed) ---
            match a.auto_open_or_close(db).await {
                Ok(Some(_new_status)) => {
                    // refresh `a` so we return the updated status
                    if let Ok(Some(refreshed)) = assignment::Entity::find_by_id(a.id).one(db).await
                    {
                        a = refreshed;
                    }
                }
                Ok(None) => { /* no change */ }
                Err(e) => {
                    // non-fatal; continue with the current record
                    eprintln!("auto_open_or_close failed: {e}");
                }
            }

            let files_res = assignment_file::Entity::find()
                .filter(assignment_file::Column::AssignmentId.eq(a.id))
                .all(db)
                .await;

            match files_res {
                Ok(files) => {
                    let converted_files: Vec<File> = files.into_iter().map(File::from).collect();

                    let mut best_mark = None;
                    let mut attempts: Option<AttemptsInfo> = None;

                    // --- NEW: compute student-safe policy (works for any caller) ---
                    let cfg = a.config().unwrap_or_else(ExecutionConfig::default_config);
                    let policy = AssignmentPolicy {
                        allow_practice_submissions: cfg.marking.allow_practice_submissions,
                        submission_mode: cfg.project.submission_mode,
                        grading_policy: cfg.marking.grading_policy,
                        limit_attempts: cfg.marking.limit_attempts,
                        pass_mark: cfg.marking.pass_mark,
                    };
                    // ---------------------------------------------------------------

                    // only if user is a student in this module
                    if let Ok(true) = UserModuleRoleService::is_in_role(
                        user.sub,
                        module_id,
                        "student".to_string(),
                    )
                    .await
                    {
                        if let Ok(Some(sub)) =
                            AssignmentSubmissionService::get_best_for_user(assignment_id, user.sub)
                                .await
                        {
                            best_mark = Some(BestMark {
                                earned: sub.earned,
                                total: sub.total,
                                attempt: sub.attempt,
                                submission_id: sub.id,
                            });
                        }

                        if let Ok(summary) = a.attempts_summary_for_user(db, user_id).await {
                            let can_submit = a.can_submit(db, user_id).await.unwrap_or(false);
                            let (max_opt, remaining_opt) = if summary.limit_attempts {
                                (Some(summary.max), Some(summary.remaining))
                            } else {
                                (None, None)
                            };

                            attempts = Some(AttemptsInfo {
                                used: summary.used,
                                max: max_opt,
                                remaining: remaining_opt,
                                can_submit,
                                limit_attempts: summary.limit_attempts,
                            });
                        }
                    }

                    let response = AssignmentFileResponse {
                        assignment: AssignmentResponse::from(a),
                        files: converted_files,
                        best_mark,
                        attempts,
                        policy: Some(policy),
                    };

                    (
                        StatusCode::OK,
                        Json(ApiResponse::success(
                            response,
                            "Assignment retrieved successfully",
                        )),
                    )
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<AssignmentFileResponse>::error(&format!(
                        "Failed to retrieve files: {}",
                        e
                    ))),
                ),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<AssignmentFileResponse>::error(
                "Assignment not found",
            )),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignmentFileResponse>::error(&format!(
                "An error occurred: {}",
                e
            ))),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct FilterReq {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub sort: Option<String>,
    pub query: Option<String>,
    pub name: Option<String>,
    pub assignment_type: Option<String>,
    pub available_before: Option<String>,
    pub available_after: Option<String>,
    pub due_before: Option<String>,
    pub due_after: Option<String>,
}

#[derive(Serialize)]
pub struct FilterResponse {
    pub assignments: Vec<AssignmentResponse>,
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
}

impl FilterResponse {
    fn new(assignments: Vec<AssignmentResponse>, page: u64, per_page: u64, total: u64) -> Self {
        Self {
            assignments,
            page,
            per_page,
            total,
        }
    }
}

/// GET /api/modules/{module_id}/assignments
///
/// Retrieve a paginated and optionally filtered list of assignments for a module. Accessible to users assigned to the module.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module to retrieve assignments from
///
/// ### Query Parameters
/// - `page` (optional, i32): Page number for pagination. Defaults to 1, minimum value is 1
/// - `per_page` (optional, i32): Number of items per page. Defaults to 20, maximum is 100, minimum is 1
/// - `sort` (optional, string): Comma-separated list of fields to sort by. Prefix with `-` for descending order (e.g., `-due_date`)
/// - `query` (optional, string): Case-insensitive substring match applied to both `name` and `description`
/// - `name` (optional, string): Case-insensitive filter to match assignment names
/// - `assignment_type` (optional, string): Filter by assignment type ("Assignment" or "Practical")
/// - `available_before` (optional, string): Filter assignments available before this date/time (ISO 8601)
/// - `available_after` (optional, string): Filter assignments available after this date/time (ISO 8601)
/// - `due_before` (optional, string): Filter assignments due before this date/time (ISO 8601)
/// - `due_after` (optional, string): Filter assignments due after this date/time (ISO 8601)
///
/// **Allowed sort fields:** `name`, `description`, `assignment_type`, `status`, `available_from`, `due_date`, `created_at`, `updated_at`
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "message": "Assignments retrieved successfully",
///   "data": {
///     "assignments": [
///       {
///         "id": 123,
///         "module_id": 456,
///         "name": "Assignment 1",
///         "description": "This is a sample assignment",
///         "assignment_type": "Assignment",
///         "available_from": "2024-01-01T00:00:00Z",
///         "due_date": "2024-01-31T23:59:59Z",
///         "created_at": "2024-01-01T00:00:00Z",
///         "updated_at": "2024-01-01T00:00:00Z"
///       }
///     ],
///     "page": 1,
///     "per_page": 20,
///     "total": 1
///   }
/// }
/// ```
///
/// - `400 Bad Request`
/// ```json
/// {
///   "success": false,
///   "message": "Invalid field used" // or "Invalid assignment_type"
/// }
/// ```
///
/// - `500 Internal Server Error`
/// ```json
/// {
///   "success": false,
///   "message": "<database error details>"
/// }
/// ```
pub async fn get_assignments(
    Path(module_id): Path<i64>,
    Query(query): Query<FilterReq>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100).max(1);
    let sort = query.sort.clone();

    let mut filters = vec![FilterParam::eq("module_id", module_id)];
    let mut queries = Vec::new();

    if let Some(query_text) = query.query {
        queries.push(QueryParam::new(
            vec!["name".to_string(), "description".to_string()],
            query_text,
        ));
    }

    if let Some(name) = query.name {
        filters.push(FilterParam::like("name", name));
    }

    if let Some(assignment_type) = query.assignment_type {
        filters.push(FilterParam::eq("assignment_type", assignment_type));
    }

    if let Some(available_before) = query.available_before {
        if let Ok(date) = DateTime::parse_from_rfc3339(&available_before) {
            filters.push(FilterParam::lt("available_from", date.with_timezone(&Utc)));
        } else {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error("Invalid available_before date format")),
            );
        }
    }

    if let Some(available_after) = query.available_after {
        if let Ok(date) = DateTime::parse_from_rfc3339(&available_after) {
            filters.push(FilterParam::gt("available_from", date.with_timezone(&Utc)));
        } else {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error("Invalid available_after date format")),
            );
        }
    }

    if let Some(due_before) = query.due_before {
        if let Ok(date) = DateTime::parse_from_rfc3339(&due_before) {
            filters.push(FilterParam::lt("due_date", date.with_timezone(&Utc)));
        } else {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error("Invalid due_before date format")),
            );
        }
    }

    if let Some(due_after) = query.due_after {
        if let Ok(date) = DateTime::parse_from_rfc3339(&due_after) {
            filters.push(FilterParam::gt("due_date", date.with_timezone(&Utc)));
        } else {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error("Invalid due_after date format")),
            );
        }
    }

    let (assignments, total) =
        match AssignmentService::filter(&filters, &queries, page, per_page, sort).await {
            Ok(result) => result,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(format!("Database error: {}", e))),
                );
            }
        };

    let assignments: Vec<AssignmentResponse> = assignments
        .into_iter()
        .map(AssignmentResponse::from)
        .collect();

    let response = FilterResponse::new(assignments, page, per_page, total);
    (
        StatusCode::OK,
        Json(ApiResponse::success(
            response,
            "Assignments retrieved successfully",
        )),
    )
}

pub fn is_late(submission: DateTime<Utc>, due_date: DateTime<Utc>) -> bool {
    submission > due_date
}

#[derive(Debug, Serialize)]
pub struct AssignmentReadiness {
    pub submission_mode: SubmissionMode,
    pub config_present: bool,
    pub tasks_present: bool,
    pub main_present: bool,
    pub interpreter_present: bool,
    pub memo_present: bool,
    pub makefile_present: bool,
    pub memo_output_present: bool,
    pub mark_allocator_present: bool,
    pub is_ready: bool,
}

/// GET /api/modules/:module_id/assignments/:assignment_id/readiness
///
/// Retrieve a detailed readiness report for a specific assignment.
///
/// The report includes boolean flags indicating whether each component is present
/// and the resolved `submission_mode` from `config.json`. Readiness is **conditional**:
/// - If `submission_mode` is **manual** → a **main** file must be present.
/// - If `submission_mode` is **gatlam** → an **interpreter** must be present.
/// - Other modes (e.g., `rng`, `codecoverage`) do not require main/interpreter.
///
/// This endpoint is useful to check if an assignment is fully set up and eligible
/// to transition from `Setup` to `Ready`.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment.
/// - `assignment_id` (i64): The ID of the assignment to check readiness for.
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "message": "Assignment readiness checked successfully",
///   "data": {
///     "submission_mode": "manual",
///     "config_present": true,
///     "tasks_present": true,
///     "main_present": true,
///     "interpreter_present": false,
///     "memo_present": true,
///     "makefile_present": true,
///     "memo_output_present": true,
///     "mark_allocator_present": true,
///     "is_ready": true
///   }
/// }
/// ```
///
/// - `500 Internal Server Error`
/// ```json
/// {
///   "success": false,
///   "message": "Failed to compute readiness: <error details>"
/// }
/// ```
///
pub async fn get_assignment_readiness(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> (StatusCode, Json<ApiResponse<AssignmentReadiness>>) {
    match AssignmentService::compute_readiness_report(module_id, assignment_id).await {
        Ok(report) => {
            // If fully ready, try to flip Setup → Ready (best-effort)
            if report.is_ready() {
                if let Err(e) =
                    AssignmentService::try_transition_to_ready(module_id, assignment_id).await
                {
                    tracing::warn!(
                        "Failed to transition assignment {} to Ready: {:?}",
                        assignment_id,
                        e
                    );
                }
            }

            let response = AssignmentReadiness {
                submission_mode: report.submission_mode,
                config_present: report.config_present,
                tasks_present: report.tasks_present,
                main_present: report.main_present,
                interpreter_present: report.interpreter_present,
                memo_present: report.memo_present,
                makefile_present: report.makefile_present,
                memo_output_present: report.memo_output_present,
                mark_allocator_present: report.mark_allocator_present,
                is_ready: report.is_ready(),
            };

            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "Assignment readiness checked successfully",
                )),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignmentReadiness>::error(&format!(
                "Failed to compute readiness: {}",
                e
            ))),
        ),
    }
}
