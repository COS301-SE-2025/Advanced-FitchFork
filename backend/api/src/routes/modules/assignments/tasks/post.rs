use crate::response::ApiResponse;
use crate::routes::modules::assignments::tasks::common::TaskResponse;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use db::models::assignment_task::{ActiveModel, Column, Entity, TaskType};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use util::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    /// Unique sequential number within the assignment
    task_number: i64,
    /// Short descriptive name
    name: String,
    /// Command to execute
    command: String,
    /// Optional; defaults to "normal". One of: "normal" | "coverage" | "valgrind".
    #[serde(default)]
    task_type: Option<TaskType>,
}

/// POST /api/modules/{module_id}/assignments/{assignment_id}/tasks
///
/// Create a new task for a given assignment. Accessible to users with Lecturer or Admin roles.
///
/// Each task must have a unique `task_number` within the assignment. The `name` field defines a short,
/// human-readable title, while `command` defines how the task is executed during evaluation.
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
///   "task_type": "normal"
/// }
/// ```
///
/// #### `task_type`
/// - `"normal"`: Regular task (default)
/// - `"coverage"`: Code coverage task (special handling)
/// - `"valgrind"`: Memory leak test (Valgrind)
///
/// ### Example Requests
/// Normal:
/// ```bash
/// curl -X POST http://localhost:3000/api/modules/1/assignments/2/tasks \
///   -H "Authorization: Bearer <token>" \
///   -H "Content-Type: application/json" \
///   -d '{"task_number":1,"name":"Build","command":"cargo build"}'
/// ```
///
/// Coverage:
/// ```bash
/// curl -X POST http://localhost:3000/api/modules/1/assignments/2/tasks \
///   -H "Authorization: Bearer <token>" \
///   -H "Content-Type: application/json" \
///   -d '{"task_number":2,"name":"Coverage run","command":"cargo llvm-cov --no-report","task_type":"coverage"}'
/// ```
///
/// Valgrind:
/// ```bash
/// curl -X POST http://localhost:3000/api/modules/1/assignments/2/tasks \
///   -H "Authorization: Bearer <token>" \
///   -H "Content-Type: application/json" \
///   -d '{"task_number":3,"name":"Memcheck","command":"valgrind --leak-check=full ./app","task_type":"valgrind"}'
/// ```
///
/// ### Success Response (201 Created)
/// ```json
/// {
///   "success": true,
///   "message": "Task created successfully",
///   "data": {
///     "id": 123,
///     "task_number": 3,
///     "name": "Memcheck",
///     "command": "valgrind --leak-check=full ./app",
///     "task_type": "valgrind",
///     "created_at": "2025-05-29T00:00:00Z",
///     "updated_at": "2025-05-29T00:00:00Z"
///   }
/// }
/// ```
///
/// ### Error Responses
/// - 422: `"Invalid task_number, name, or command"`
/// - 422: `"task_number must be unique"`
/// - 500: `"Failed to create task"`
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

    // Ensure task_number uniqueness within the assignment
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
    let task_type = payload.task_type.unwrap_or(TaskType::Normal);

    let new_task = ActiveModel {
        assignment_id: sea_orm::ActiveValue::Set(assignment_id),
        task_number: sea_orm::ActiveValue::Set(payload.task_number),
        name: sea_orm::ActiveValue::Set(payload.name.clone()),
        command: sea_orm::ActiveValue::Set(payload.command.clone()),
        task_type: sea_orm::ActiveValue::Set(task_type),
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
                task_type: task.task_type,
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
