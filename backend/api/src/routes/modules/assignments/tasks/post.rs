use crate::response::ApiResponse;
use crate::routes::modules::assignments::tasks::common::TaskResponse;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use db::models::assignment_task::{ActiveModel, Column, Entity};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use util::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    task_number: i64,
    name: String,
    command: String,
    /// Optional; defaults to false. When true, this task is a **coverage-type** task.
    code_coverage: Option<bool>,
}

/// POST /api/modules/{module_id}/assignments/{assignment_id}/tasks
///
/// Create a new task for a given assignment. Accessible to users with Lecturer or Admin roles
/// assigned to the module.
///
/// Each task must have a unique `task_number` within the assignment. The `name` field defines a short,
/// human-readable title for the task, while the `command` field defines how the task will be executed
/// during evaluation (e.g., test commands, build commands).
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to add the task to
///
/// ### Request Body
/// ```json
/// {
///   "task_number": 1,
///   "name": "Unit Tests",
///   "command": "cargo test --lib",
///   "code_coverage": false
/// }
/// ```
///
/// ### Request Body Fields
/// - `task_number` (i64, required): Unique sequential number for the task within the assignment
/// - `name` (string, required): Short descriptive name for the task (e.g., "Compile", "Unit Tests")
/// - `command` (string, required): Command to execute for this task (e.g., test commands, build scripts)
/// - `code_coverage` (boolean, optional, default: false): Marks this task as a **coverage-type** task
///
/// ### Example Request
/// ```bash
/// curl -X POST http://localhost:3000/api/modules/1/assignments/2/tasks \
///   -H "Authorization: Bearer <token)" \
///   -H "Content-Type: application/json" \
///   -d '{
///     "task_number": 1,
///     "name": "Coverage run",
///     "command": "cargo llvm-cov --no-report",
///     "code_coverage": true
///   }'
/// ```
///
/// ### Success Response (201 Created)
/// ```json
/// {
///   "success": true,
///   "message": "Task created successfully",
///   "data": {
///     "id": 123,
///     "task_number": 1,
///     "name": "Coverage run",
///     "command": "cargo llvm-cov --no-report",
///     "code_coverage": true,
///     "created_at": "2024-01-01T00:00:00Z",
///     "updated_at": "2024-01-01T00:00:00Z"
///   }
/// }
/// ```
///
/// ### Error Responses
///
/// **400 Bad Request** - Invalid JSON body
/// ```json
/// { "success": false, "message": "Invalid JSON body" }
/// ```
///
/// **403 Forbidden** - Insufficient permissions
/// ```json
/// { "success": false, "message": "Access denied" }
/// ```
///
/// **404 Not Found** - Assignment or module not found
/// ```json
/// { "success": false, "message": "Assignment or module not found" }
/// ```
///
/// **422 Unprocessable Entity** - Validation errors
/// ```json
/// { "success": false, "message": "Invalid task_number, name, or command" }
/// ```
/// or
/// ```json
/// { "success": false, "message": "task_number must be unique" }
/// ```
///
/// **500 Internal Server Error** - Database or server error
/// ```json
/// { "success": false, "message": "Failed to create task" }
/// ```
///
/// ### Validation Rules
/// - `task_number` must be greater than 0
/// - `name` must not be empty or whitespace-only
/// - `command` must not be empty or whitespace-only
/// - `task_number` must be unique within the assignment
/// - Assignment must exist and belong to the specified module
///
/// ### Notes
/// - Tasks are executed in order of their `task_number` during assignment evaluation
/// - `code_coverage: true` designates a **coverage-type** task (the evaluator may apply coverage-specific handling)
/// - Task creation is restricted to users with appropriate module permissions
pub async fn create_task(
    State(app_state): State<AppState>,
    Path((_, assignment_id)): Path<(i64, i64)>,
    Json(payload): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    let db = app_state.db();

    // Validation: task_number > 0, name non-empty, command non-empty
    if payload.task_number <= 0
        || payload.name.trim().is_empty()
        || payload.command.trim().is_empty()
    {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<()>::error(
                "Invalid task_number, name, or command",
            )),
        )
            .into_response();
    }

    // Ensure task_number uniqueness
    match Entity::find()
        .filter(Column::AssignmentId.eq(assignment_id))
        .filter(Column::TaskNumber.eq(payload.task_number))
        .one(db)
        .await
    {
        Ok(Some(_)) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiResponse::<()>::error("task_number must be unique")),
            )
                .into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to create task")),
            )
                .into_response();
        }
    }

    let now = Utc::now();
    let new_task = ActiveModel {
        assignment_id: sea_orm::ActiveValue::Set(assignment_id),
        task_number: sea_orm::ActiveValue::Set(payload.task_number),
        name: sea_orm::ActiveValue::Set(payload.name.clone()),
        command: sea_orm::ActiveValue::Set(payload.command.clone()),
        code_coverage: sea_orm::ActiveValue::Set(payload.code_coverage.unwrap_or(false)),
        created_at: sea_orm::ActiveValue::Set(now),
        updated_at: sea_orm::ActiveValue::Set(now),
        ..Default::default()
    };

    match new_task.insert(db).await {
        Ok(task) => {
            let response = TaskResponse {
                id: task.id,
                task_number: task.task_number,
                name: task.name,
                command: task.command,
                code_coverage: task.code_coverage,
                created_at: task.created_at.to_rfc3339(),
                updated_at: task.updated_at.to_rfc3339(),
            };

            (
                StatusCode::CREATED,
                Json(ApiResponse::success(response, "Task created successfully")),
            )
                .into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to create task")),
        )
            .into_response(),
    }
}
