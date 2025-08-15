use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::types::JsonValue;
use serde_json::json;
use db::models::assignment;
use crate::response::ApiResponse;
use super::common::BulkDeleteRequest;

/// DELETE /api/modules/:module_id/assignments/:assignment_id
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
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let db = db::get_connection().await;

    match assignment::Model::delete(db, assignment_id as i32, module_id as i32).await {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "message": format!("Assignment {} deleted successfully", assignment_id),
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

/// DELETE /api/modules/:module_id/assignments/bulk
///
/// Bulk delete multiple assignments by ID within a module.
/// Only accessible by lecturers or admins assigned to the module.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignments
///
/// ### Request Body (JSON)
/// ```json
/// {
///   "assignment_ids": [123, 124, 125]
/// }
/// ```
///
/// ### Success Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "data": {
///     "deleted": 2,
///     "failed": [
///       { "id": 125, "error": "Assignment 125 in module 1 not found" }
///     ]
///   },
///   "message": "Deleted 2/3 assignments"
/// }
/// ```
pub async fn bulk_delete_assignments(
    Path(module_id): Path<i64>,
    Json(req): Json<BulkDeleteRequest>,
) -> impl IntoResponse {
    let db = db::get_connection().await;

    if req.assignment_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<JsonValue>::error("No assignment IDs provided")),
        );
    }

    let mut deleted_count = 0;
    let mut failed: Vec<JsonValue> = Vec::new();

    for &id in &req.assignment_ids {
        match assignment::Model::delete(db, id as i32, module_id as i32).await {
            Ok(_) => deleted_count += 1,
            Err(e) => {
                failed.push(json!({
                    "id": id,
                    "error": e.to_string()
                }));
            }
        }
    }

    let message = format!(
        "Deleted {}/{} assignments",
        deleted_count,
        req.assignment_ids.len()
    );

    let data = json!({
        "deleted": deleted_count,
        "failed": failed
    });

    let response = ApiResponse::success(data, message);

    (
        StatusCode::OK,
        Json(response),
    )
}