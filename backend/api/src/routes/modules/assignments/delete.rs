//! Assignment deletion routes.
//!
//! Provides endpoints for deleting single or multiple assignments within a module.
//!
//! - `DELETE /api/modules/{module_id}/assignments/{assignment_id}`  
//!   Deletes a single assignment along with its associated files and folder.
//!
//! - `DELETE /api/modules/{module_id}/assignments/bulk`  
//!   Deletes multiple assignments in a module using a JSON array of assignment IDs.
//!
//! **Access Control:** Only lecturers or admins assigned to the module can perform deletions.
//!
//! **Responses:** JSON-wrapped `ApiResponse` indicating success, number of deletions, or detailed errors.

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use serde::{Serialize, Deserialize};
use crate::response::ApiResponse;
//use super::common::{BulkDeleteRequest, BulkDeleteResponse};
use services::service::Service;
use services::assignment::AssignmentService;

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
    match AssignmentService::delete_by_id(module_id).await {
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

#[derive(Deserialize)]
pub struct BulkDeleteRequest {
    pub assignment_ids: Vec<i64>,
}

#[derive(Serialize)]
pub struct BulkDeleteResult {
    pub deleted: usize,
    pub failed: Vec<FailedDelete>,
}

#[derive(Serialize)]
pub struct FailedDelete {
    pub id: i64,
    pub error: String,
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
    Path(_): Path<i64>,
    Json(req): Json<BulkDeleteRequest>,
) -> impl IntoResponse {
    if req.assignment_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<BulkDeleteResult>::error("No assignment IDs provided")),
        );
    }

    let mut deleted_count = 0;
    let mut failed = Vec::new();

    for &id in &req.assignment_ids {
        match AssignmentService::delete_by_id(id).await {
            Ok(_) => deleted_count += 1,
            Err(e) => {
                failed.push(FailedDelete {
                    id,
                    error: format!("Failed to delete assignment: {}", e),
                });
            }
        }
    }
    
    let result = BulkDeleteResult {
        deleted: deleted_count,
        failed,
    };

    let message = format!(
        "Deleted {}/{} assignments",
        deleted_count,
        req.assignment_ids.len()
    );

    (
        StatusCode::OK,
        Json(ApiResponse::success(result, message)),
    )
}