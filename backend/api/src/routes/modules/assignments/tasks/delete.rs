use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use sea_orm::{
    ColumnTrait,
    DbErr,
    EntityTrait,
    QueryFilter,
};

use db::{
    connect,
    models::{assignment, assignment_task},
};

use crate::response::ApiResponse;

/// DELETE /api/modules/{module_id}/assignments/{assignment_id}/tasks/{task_id}
///
/// Delete a specific task from an assignment. Only accessible by lecturers or admins assigned to the module.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment containing the task
/// - `task_id` (i64): The ID of the task to delete
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "message": "Task deleted successfully"
/// }
/// ```
///
/// - `404 Not Found`
/// ```json
/// {
///   "success": false,
///   "message": "Assignment or module not found" // or "Task not found"
/// }
/// ```
///
/// - `500 Internal Server Error`
/// ```json
/// {
///   "success": false,
///   "message": "Database error" // or "Failed to delete task"
/// }
/// ```
///
pub async fn delete_task(
    Path((module_id, assignment_id, task_id)): Path<(i64, i64, i64)>,
) -> impl IntoResponse {
    let db = connect().await;

    let assignment = assignment::Entity::find()
        .filter(assignment::Column::Id.eq(assignment_id as i32))
        .filter(assignment::Column::ModuleId.eq(module_id as i32))
        .one(&db)
        .await;

    match assignment {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Assignment or module not found")),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error")),
            )
                .into_response();
        }
    }

    match assignment_task::Model::delete(&db, task_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success((), "Task deleted successfully")),
        )
            .into_response(),
        Err(DbErr::RecordNotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Task not found")),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to delete task")),
        )
            .into_response(),
    }
}