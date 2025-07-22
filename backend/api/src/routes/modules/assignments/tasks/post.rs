use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, DatabaseConnection};
use serde::Deserialize;
use crate::response::ApiResponse;
use db::models::assignment_task::{ActiveModel, Column, Entity};
use crate::routes::modules::assignments::tasks::common::TaskResponse;

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    task_number: i64,
    command: String,
}

/// POST /api/modules/{module_id}/assignments/{assignment_id}/tasks
///
/// Create a new task for a given assignment. Accessible to users with Lecturer or Admin roles
/// assigned to the module.
///
/// Each task must have a unique `task_number` within the assignment. The `command` field defines
/// how the task will be executed during evaluation (e.g., test commands, build commands).
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to add the task to
///
/// ### Request Body
/// ```json
/// {
///   "task_number": 1,
///   "command": "cargo test --lib"
/// }
/// ```
///
/// ### Request Body Fields
/// - `task_number` (i64, required): Unique sequential number for the task within the assignment
/// - `command` (string, required): Command to execute for this task (e.g., test commands, build scripts)
///
/// ### Example Request
/// ```bash
/// curl -X POST http://localhost:3000/api/modules/1/assignments/2/tasks \
///   -H "Authorization: Bearer <token>" \
///   -H "Content-Type: application/json" \
///   -d '{
///     "task_number": 1,
///     "command": "cargo test --lib"
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
///     "command": "cargo test --lib",
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
/// **404 Not Found** - Assignment or module not found
/// ```json
/// {
///   "success": false,
///   "message": "Assignment or module not found"
/// }
/// ```
///
/// **422 Unprocessable Entity** - Validation errors
/// ```json
/// {
///   "success": false,
///   "message": "Invalid task_number or command"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "task_number must be unique"
/// }
/// ```
///
/// **500 Internal Server Error** - Database or server error
/// ```json
/// {
///   "success": false,
///   "message": "Failed to create task"
/// }
/// ```
///
/// ### Validation Rules
/// - `task_number` must be greater than 0
/// - `command` must not be empty or whitespace-only
/// - `task_number` must be unique within the assignment
/// - Assignment must exist and belong to the specified module
///
/// ### Notes
/// - Tasks are executed in order of their `task_number` during assignment evaluation
/// - The `command` field supports any shell command that can be executed in the evaluation environment
/// - Task creation is restricted to users with appropriate module permissions
pub async fn create_task(
    State(db): State<DatabaseConnection>,
    Path((_, assignment_id)): Path<(i64, i64)>,
    Json(payload): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    if payload.task_number <= 0 || payload.command.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<()>::error("Invalid task_number or command")),
        )
            .into_response();
    }

    let exists = Entity::find()
        .filter(Column::AssignmentId.eq(assignment_id))
        .filter(Column::TaskNumber.eq(payload.task_number))
        .one(&db)
        .await;

    if let Ok(Some(_)) = exists {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<()>::error("task_number must be unique")),
        )
            .into_response();
    }

    let now = Utc::now();
    let new_task = ActiveModel {
        assignment_id: sea_orm::ActiveValue::Set(assignment_id),
        task_number: sea_orm::ActiveValue::Set(payload.task_number),
        command: sea_orm::ActiveValue::Set(payload.command.clone()),
        created_at: sea_orm::ActiveValue::Set(now.clone()),
        updated_at: sea_orm::ActiveValue::Set(now.clone()),
        ..Default::default()
    };

    match new_task.insert(&db).await {
        Ok(task) => {
            let response = TaskResponse {
                id: task.id,
                task_number: task.task_number,
                name: task.name,
                command: task.command,
                created_at: task.created_at,
                updated_at: task.updated_at,
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