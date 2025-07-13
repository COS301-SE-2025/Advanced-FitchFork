//! Task Edit Endpoint
//!
//! This module provides the endpoint handler for editing the command of a specific assignment task within a module. It validates the existence and relationships of the module, assignment, and task, and updates the task's command in the database. The endpoint returns detailed information about the updated task or appropriate error responses.

use axum::{extract::{Path, Json}, http::StatusCode, response::IntoResponse};
use db::connect;
use db::models::{assignment_task, assignment, module};
use serde::Deserialize;
use crate::response::ApiResponse;
use sea_orm::{EntityTrait, DbErr};
use crate::routes::modules::assignments::tasks::common::TaskResponse;

/// The request payload for editing a task's command.
#[derive(Deserialize)]
pub struct EditTaskRequest {
    /// The new command string for the task. Must be non-empty.
    command: String,
    /// The new name for the task. Must be non-empty.
    name: String,
}

/// PUT /api/modules/:module_id/assignments/:assignment_id/tasks/:task_id
///
/// Edit the command of a specific task within an assignment. Accessible to users with Lecturer or Admin roles
/// assigned to the module.
///
/// This endpoint allows updating the command that will be executed during task evaluation. The command
/// can be any shell command that can be executed in the evaluation environment.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment containing the task
/// - `task_id` (i64): The ID of the task to edit
///
/// ### Request Body
/// ```json
/// {
///   "command": "cargo test --lib --release"
/// }
/// ```
///
/// ### Request Body Fields
/// - `command` (string, required): The new command to execute for this task (e.g., test commands, build scripts)
///
/// ### Example Request
/// ```bash
/// curl -X PUT http://localhost:3000/api/modules/1/assignments/2/tasks/3 \
///   -H "Authorization: Bearer <token>" \
///   -H "Content-Type: application/json" \
///   -d '{
///     "command": "cargo test --lib --release"
///   }'
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
///     "command": "cargo test --lib --release",
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
///   "message": "'command' must be a non-empty string"
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
/// - `command` must not be empty or whitespace-only
/// - Module must exist
/// - Assignment must exist and belong to the specified module
/// - Task must exist and belong to the specified assignment
///
/// ### Notes
/// - Only the command field can be updated; task_number and other fields remain unchanged
/// - The updated task will be used in future assignment evaluations
/// - Task editing is restricted to users with appropriate module permissions
/// - The `updated_at` timestamp is automatically set when the task is modified
pub async fn edit_task(
    Path((module_id, assignment_id, task_id)): Path<(i64, i64, i64)>,
    Json(payload): Json<EditTaskRequest>,
) -> impl IntoResponse {
    if payload.command.trim().is_empty() || payload.name.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<()>::error("'name' and 'command' must be non-empty strings")),
        ).into_response();
    }

    let db = connect().await;
    let module_exists = match module::Entity::find_by_id(module_id).one(&db).await {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error retrieving module")),
            ).into_response();
        }
    };
    
    if !module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        ).into_response();
    }

    let assignment_model = match assignment::Entity::find_by_id(assignment_id).one(&db).await {
        Ok(Some(a)) => a,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Assignment not found")),
            ).into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error retrieving assignment")),
            ).into_response();
        }
    };

    if assignment_model.module_id != module_id {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Assignment does not belong to this module")),
        ).into_response();
    }

    let task = match assignment_task::Entity::find_by_id(task_id).one(&db).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Task not found")),
            ).into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error retrieving task")),
            ).into_response();
        }
    };

    if task.assignment_id != assignment_id {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Task does not belong to this assignment")),
        ).into_response();
    }

    let updated = match assignment_task::Model::edit_command_and_name(&db, task_id, &payload.name, &payload.command).await {
        Ok(t) => t,
        Err(DbErr::RecordNotFound(_)) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Task not found for update")),
            ).into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to update task")),
            ).into_response();
        }
    };

    let resp = TaskResponse {
        id: updated.id,
        task_number: updated.task_number,
        name: updated.name,
        command: updated.command,
        created_at: updated.created_at,
        updated_at: updated.updated_at,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(resp, "Task updated successfully")),
    ).into_response()
}