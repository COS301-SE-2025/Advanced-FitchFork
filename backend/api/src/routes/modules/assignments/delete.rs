use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{DatabaseConnection, DbErr};
use serde_json::json;
use db::{
    models::{assignment::{self}},
};

/// DELETE /api/modules/{module_id}/assignments/{assignment_id}
///
/// Delete a specific assignment and its associated files and folder.
/// Only accessible by lecturers or admins assigned to the module.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to delete
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "message": "Assignment 123 deleted successfully"
/// }
/// ```
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "message": "No assignment found with ID 123 in module 456"
/// }
/// ```
///
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "message": "Database error details"
/// }
/// ```
pub async fn delete_assignment(
    State(db): State<DatabaseConnection>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    match assignment::Model::delete(&db, assignment_id as i32, module_id as i32).await {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "message": format!("Assignment {} deleted successfully", assignment_id),
            })),
        ),
        Err(DbErr::RecordNotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "message": format!("No assignment found with ID {} in module {}", assignment_id, module_id),
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "message": e.to_string(),
            })),
        ),
    }
}