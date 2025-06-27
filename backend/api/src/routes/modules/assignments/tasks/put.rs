//! Task Edit Endpoint
//!
//! This module provides the endpoint handler for editing the command of a specific assignment task within a module. It validates the existence and relationships of the module, assignment, and task, and updates the task's command in the database. The endpoint returns detailed information about the updated task or appropriate error responses.

use axum::{extract::{Path, Json}, http::StatusCode, response::IntoResponse};
use db::connect;
use db::models::{assignment_task, assignment, module};
use serde::Deserialize;
use crate::response::ApiResponse;
use sea_orm::{EntityTrait, DbErr};

/// The request payload for editing a task's command.
#[derive(Deserialize)]
pub struct EditTaskRequest {
    /// The new command string for the task. Must be non-empty.
    command: String,
    /// The new name for the task. Must be non-empty.
    name: String,
}

/// The response structure for a task, including its updated details.
#[derive(serde::Serialize)]
struct TaskResponse {
    /// The unique database ID of the task.
    id: i64,
    /// The task's number (as used in the allocator and memo files).
    task_number: i64,
    /// The name for the task.
    name: String,
    /// The command associated with this task.
    command: String,
    /// The creation timestamp of the task (RFC3339 format).
    created_at: String,
    /// The last update timestamp of the task (RFC3339 format).
    updated_at: String,
}

/// Edits the command of a specific task within an assignment and module.
///
/// This handler performs the following steps:
/// 1. Validates that the provided command is non-empty.
/// 2. Checks for the existence of the module, assignment, and task in the database.
/// 3. Ensures the assignment belongs to the module, and the task belongs to the assignment.
/// 4. Updates the command of the specified task in the database.
/// 5. Returns the updated task details in the response.
///
/// # Path Parameters
/// - `module_id`: The ID of the module.
/// - `assignment_id`: The ID of the assignment.
/// - `task_id`: The ID of the task.
///
/// # Request Body
/// - `command`: The new command string for the task (must be non-empty).
///
/// # Returns
/// - `200 OK` with updated task information if successful.
/// - `404 Not Found` if the module, assignment, or task does not exist or does not belong.
/// - `422 Unprocessable Entity` if the command is empty.
/// - `500 Internal Server Error` for database errors.
///
/// The response includes the updated task details.
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
        created_at: updated.created_at.to_rfc3339(),
        updated_at: updated.updated_at.to_rfc3339(),
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(resp, "Task updated successfully")),
    ).into_response()
}
