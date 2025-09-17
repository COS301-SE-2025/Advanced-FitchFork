//! Submission deletion routes.
//!
//! Provides endpoints for deleting single or multiple submissions within an assignment.
//!
//! - `DELETE /api/modules/{module_id}/assignments/{assignment_id}/submissions/{submission_id}`  
//!   Deletes a single submission and attempts to remove its stored file.
//!
//! - `DELETE /api/modules/{module_id}/assignments/{assignment_id}/submissions/bulk`  
//!   Deletes multiple submissions using a JSON array of submission IDs.
//!
//! **Access Control:** Only lecturers or assistant lecturers may delete submissions.
//!
//! **Responses:** JSON-wrapped `ApiResponse` indicating success, number of deletions, or detailed errors.

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::response::ApiResponse;
use services::service::Service;
use services::assignment_submission::AssignmentSubmissionService;

/// DELETE /api/modules/:module_id/assignments/:assignment_id/submissions/:submission_id
///
/// Delete a specific submission and (best-effort) its stored file.
/// Only accessible by lecturers or assistant lecturers.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment
/// - `submission_id` (i64): The ID of the submission to delete
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "message": "Submission 987 deleted successfully"
/// }
/// ```
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "message": "No submission 987 found for assignment 123"
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
pub async fn delete_submission(
    Path((_, _, submission_id)): Path<(i64, i64, i64)>,
) -> impl IntoResponse {
    if let Err(e) = AssignmentSubmissionService::delete_file_only(submission_id).await {
        eprintln!(
            "delete_submission: failed to remove file for submission {}: {}",
            submission_id, e
        );
    }

    match AssignmentSubmissionService::delete_by_id(submission_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success_without_data(
                format!("Submission {} deleted successfully", submission_id)
            )),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct BulkDeleteSubmissionsRequest {
    pub submission_ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
pub struct BulkDeleteResponse {
    pub deleted: usize,
    pub failed: Vec<FailedDeletion>,
}

#[derive(Debug, Serialize)]
pub struct FailedDeletion {
    pub id: i64,
    pub error: String,
}

/// DELETE /api/modules/:module_id/assignments/:assignment_id/submissions/bulk
///
/// Bulk delete multiple submissions by ID within an assignment.
/// Only accessible by lecturers or assistant lecturers.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment
///
/// ### Request Body (JSON)
/// ```json
/// {
///   "submission_ids": [981, 982, 983]
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
///       { "id": 983, "error": "No submission 983 found for assignment 123" }
///     ]
///   },
///   "message": "Deleted 2/3 submissions"
/// }
/// ```
pub async fn bulk_delete_submissions(
    Path((_, _)): Path<(i64, i64)>,
    Json(req): Json<BulkDeleteSubmissionsRequest>,
) -> impl IntoResponse {
    if req.submission_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<BulkDeleteResponse>::error("No submission IDs provided")),
        );
    }

    let mut deleted_count = 0;
    let mut failed: Vec<FailedDeletion> = Vec::new();

    for &sid in &req.submission_ids {
        if let Err(e) = AssignmentSubmissionService::delete_file_only(sid).await {
            eprintln!("bulk_delete_submissions: file removal failed for {}: {}", sid, e);
        }

        match AssignmentSubmissionService::delete_by_id(sid).await {
            Ok(_) => deleted_count += 1,
            Err(e) => failed.push(FailedDeletion { id: sid, error: e.to_string() }),
        }
    }

    let message = format!(
        "Deleted {}/{} submissions",
        deleted_count,
        req.submission_ids.len()
    );

    let data = BulkDeleteResponse {
        deleted: deleted_count,
        failed,
    };

    let response = ApiResponse::success(data, message);

    (StatusCode::OK, Json(response))
}
