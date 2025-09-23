//! Task Edit Endpoint
//!
//! This module provides the endpoint handler for editing the command of a specific assignment task within a module. It validates the existence and relationships of the module, assignment, and task, and updates the task's command in the database. The endpoint returns detailed information about the updated task or appropriate error responses.

use crate::response::ApiResponse;
use crate::routes::modules::assignments::tasks::common::TaskResponse;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use db::models::assignment_task;
use sea_orm::DbErr;
use serde::Deserialize;
use util::state::AppState;

/// The request payload for editing a task's command.
#[derive(Deserialize)]
pub struct EditTaskRequest {
    /// Optional new name
    name: Option<String>,
    /// Optional new command
    command: Option<String>,
    /// Optional toggle for coverage
    code_coverage: Option<bool>,
    /// Optional toggle for valgrind
    valgrind: Option<bool>,
}

/// PUT /api/modules/{module_id}/assignments/{assignment_id}/tasks/{task_id}
///
/// Edit one or more fields of a specific task within an assignment (supports partial updates).
/// Accessible to users with Lecturer or Admin roles assigned to the module.
///
/// This endpoint updates task metadata used during evaluation. You can change:
/// - `name` (label shown to users)
/// - `command` (shell command executed by the runner)
/// - `code_coverage` (whether this task is a **coverage-type** task)
///
/// > Note: `code_coverage: true` marks the task as a **code coverage task**. The evaluator
/// > may apply coverage-specific handling (e.g., expecting coverage artifacts). It does **not**
/// > merely “collect coverage” for arbitrary tasks; it designates the task’s type.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment containing the task
/// - `task_id` (i64): The ID of the task to edit
///
/// ### Request Body
/// Any subset of fields may be provided; at least one is required.
/// ```json
/// {
///   "name": "Unit tests",
///   "command": "cargo test --lib --release",
///   "code_coverage": true
/// }
/// ```
///
/// ### Request Body Fields
/// - `name` (string, optional): New display name for the task; if provided, must be non-empty
/// - `command` (string, optional): New command to execute; if provided, must be non-empty
/// - `code_coverage` (boolean, optional): Set whether this task is a **code coverage task**
///
/// ### Example Requests
/// Update only the command:
/// ```bash
/// curl -X PUT http://localhost:3000/api/modules/1/assignments/2/tasks/3 \
///   -H "Authorization: Bearer <token>" \
///   -H "Content-Type: application/json" \
///   -d '{"command":"cargo test --lib --release"}'
/// ```
///
/// Mark the task as a coverage-type task (or unset it):
/// ```bash
/// curl -X PUT http://localhost:3000/api/modules/1/assignments/2/tasks/3 \
///   -H "Authorization: Bearer <token)" \
///   -H "Content-Type: application/json" \
///   -d '{"code_coverage": true}'
/// ```
///
/// Rename and change command:
/// ```bash
/// curl -X PUT http://localhost:3000/api/modules/1/assignments/2/tasks/3 \
///   -H "Authorization: Bearer <token)" \
///   -H "Content-Type: application/json" \
///   -d '{"name":"Coverage run","command":"cargo llvm-cov --no-report"}'
/// ```
///
/// ### Success Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "message": "Task updated successfully",
///   "data": {
///     "id": 3,
///     "task_number": 1,
///     "name": "Coverage run",
///     "command": "cargo llvm-cov --no-report",
///     "code_coverage": true,
///     "created_at": "2024-01-01T00:00:00Z",
///     "updated_at": "2024-01-01T12:30:00Z"
///   }
/// }
/// ```
///
/// ### Error Responses
///
/// **400 Bad Request** - Invalid JSON body
/// ```json
/// {
///   "success": false,
///   "message": "Invalid JSON body"
/// }
/// ```
///
/// **403 Forbidden** - Insufficient permissions
/// ```json
/// {
///   "success": false,
///   "message": "Access denied"
/// }
/// ```
///
/// **404 Not Found** - Resource not found
/// ```json
/// {
///   "success": false,
///   "message": "Module not found"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Assignment not found"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Task not found"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Assignment does not belong to this module"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Task does not belong to this assignment"
/// }
/// ```
///
/// **422 Unprocessable Entity** - Validation error
/// ```json
/// {
///   "success": false,
///   "message": "At least one of 'name', 'command', or 'code_coverage' must be provided"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "'name' and 'command' must be non-empty if provided"
/// }
/// ```
///
/// **500 Internal Server Error** - Database or server error
/// ```json
/// {
///   "success": false,
///   "message": "Database error retrieving module"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Failed to update task"
/// }
/// ```
///
/// ### Validation Rules
/// - At least one of `name`, `command`, `code_coverage` must be provided
/// - If present, `name` and `command` must not be empty or whitespace-only
/// - Module must exist
/// - Assignment must exist and belong to the specified module
/// - Task must exist and belong to the specified assignment
///
/// ### Notes
/// - This route performs a partial update although it uses `PUT`
/// - `task_number` and other immutable fields remain unchanged
/// - Setting `code_coverage: true` designates the task as a **code coverage task**
/// - The `updated_at` timestamp is automatically set when the task is modified
pub async fn edit_task(
    State(app_state): State<AppState>,
    Path((_, _, task_id)): Path<(i64, i64, i64)>,
    Json(payload): Json<EditTaskRequest>,
) -> impl IntoResponse {
    let db = app_state.db();

    // Must provide something to change
    if payload.name.is_none() && payload.command.is_none() && payload.code_coverage.is_none() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<()>::error(
                "At least one of 'name', 'command', or 'code_coverage' must be provided",
            )),
        )
            .into_response();
    }

    // If present, basic validation
    if payload
        .name
        .as_deref()
        .map(|s| s.trim().is_empty())
        .unwrap_or(false)
        || payload
            .command
            .as_deref()
            .map(|s| s.trim().is_empty())
            .unwrap_or(false)
    {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<()>::error(
                "'name' and 'command' must be non-empty strings",
            )),
        )
            .into_response();
    }

    let updated = match assignment_task::Model::edit(
        db,
        task_id,
        payload.name.as_deref(),
        payload.command.as_deref(),
        payload.code_coverage,
        payload.valgrind,
    )
    .await
    {
        Ok(t) => t,
        Err(DbErr::RecordNotFound(_)) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Task not found")),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to update task")),
            )
                .into_response();
        }
    };

    let resp = TaskResponse {
        id: updated.id,
        task_number: updated.task_number,
        name: updated.name,
        command: updated.command,
        code_coverage: updated.code_coverage,
        created_at: updated.created_at.to_rfc3339(),
        updated_at: updated.updated_at.to_rfc3339(),
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(resp, "Task updated successfully")),
    )
        .into_response()
}
