//! Submission patch routes.
//!
//! Provides endpoints to toggle the `ignored` flag on submissions,
//! both individually and in bulk.
//!
//! - `PATCH /api/modules/{module_id}/assignments/{assignment_id}/submissions/{submission_id}/ignore`  
//!   Set/unset the `ignored` flag for a single submission.
//!
//!
//! **Access Control:** Only lecturers or assistant lecturers may perform these actions.
//!
//! **Notes:**
//! - An `ignored` submission should be excluded from grading/analytics where applicable.
//! - The endpoint validates that the submission belongs to the target assignment.

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::response::ApiResponse;
use services::service::Service;
use services::assignment_submission::{AssignmentSubmissionService, UpdateAssignmentSubmission};

#[derive(Debug, Deserialize)]
pub struct SetIgnoredReq {
    /// Desired value for the `ignored` flag.
    pub ignored: bool,
}

/// Response payload for single-item `ignored` updates.
#[derive(Debug, Serialize)]
struct SetIgnoredData {
    /// Submission ID.
    id: i64,
    /// The resulting `ignored` value after update.
    ignored: bool,
    /// RFC3339 timestamp of the update.
    updated_at: String,
}

/// PATCH /api/modules/:module_id/assignments/:assignment_id/submissions/:submission_id/ignore
///
/// Toggle the `ignored` flag for a **single** submission.  
/// Only accessible by lecturers or assistant lecturers.
///
/// # Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment
/// - `submission_id` (i64): The ID of the submission to update
///
/// # Request Body (JSON)
/// ```json
/// { "ignored": true }
/// ```
///
/// # Returns
/// - `200 OK` with the updated state
/// - `404 NOT FOUND` if the submission does not exist under the assignment
/// - `500 INTERNAL SERVER ERROR` on database errors
///
/// ## Example Success
/// ```json
/// {
///   "success": true,
///   "data": {
///     "id": 987,
///     "ignored": true,
///     "updated_at": "2025-05-29T12:34:56Z"
///   },
///   "message": "Submission ignored"
/// }
/// ```
pub async fn set_submission_ignored(
    Path((_, _, submission_id)): Path<(i64, i64, i64)>,
    Json(req): Json<SetIgnoredReq>,
) -> impl IntoResponse {
    match AssignmentSubmissionService::update(
        UpdateAssignmentSubmission {
            id: submission_id,
            ignored: Some(req.ignored),
        }
    ).await {
        Ok(updated) => {
            let data = SetIgnoredData {
                id: updated.id,
                ignored: updated.ignored,
                updated_at: updated.updated_at.to_rfc3339(),
            };
            (
                StatusCode::OK,
                Json(ApiResponse::<SetIgnoredData>::success(
                    data,
                    if req.ignored { "Submission ignored" } else { "Submission unignored" },
                )),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<SetIgnoredData>::error(format!("Failed to update: {}", e))),
        ),
    }
}