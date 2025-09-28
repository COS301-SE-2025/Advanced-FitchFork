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
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::types::JsonValue;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::response::ApiResponse;
use util::state::AppState;

use db::models::assignment_submission as submission;

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
    State(app_state): State<AppState>,
    Path((_module_id, assignment_id, submission_id)): Path<(i64, i64, i64)>,
) -> impl IntoResponse {
    let db = app_state.db();

    // Ensure the submission exists and belongs to the assignment
    let sub = match submission::Entity::find()
        .filter(submission::Column::Id.eq(submission_id))
        .filter(submission::Column::AssignmentId.eq(assignment_id))
        .one(db)
        .await
    {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "success": false,
                    "message": format!("No submission {} found for assignment {}", submission_id, assignment_id)
                })),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "message": e.to_string(),
                })),
            );
        }
    };

    // Best-effort: remove stored file first (ignore failure, log if needed)
    if let Err(e) = sub.delete_file_only() {
        eprintln!(
            "delete_submission: failed to remove file for submission {}: {}",
            submission_id, e
        );
    }

    // Delete DB row by primary key (we already validated assignment_id)
    match submission::Entity::delete_by_id(submission_id)
        .exec(db)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "message": format!("Submission {} deleted successfully", submission_id),
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

#[derive(Debug, Deserialize)]
pub struct BulkDeleteSubmissionsRequest {
    pub submission_ids: Vec<i64>,
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
    State(app_state): State<AppState>,
    Path((_module_id, assignment_id)): Path<(i64, i64)>,
    Json(req): Json<BulkDeleteSubmissionsRequest>,
) -> impl IntoResponse {
    let db = app_state.db();

    if req.submission_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<JsonValue>::error(
                "No submission IDs provided",
            )),
        );
    }

    let mut deleted_count = 0;
    let mut failed: Vec<JsonValue> = Vec::new();

    for &sid in &req.submission_ids {
        // Lookup
        match submission::Entity::find()
            .filter(submission::Column::Id.eq(sid))
            .filter(submission::Column::AssignmentId.eq(assignment_id))
            .one(db)
            .await
        {
            Ok(Some(sub)) => {
                // Remove file best-effort
                if let Err(e) = sub.delete_file_only() {
                    eprintln!(
                        "bulk_delete_submissions: file removal failed for {}: {}",
                        sid, e
                    );
                }

                // Delete row
                match submission::Entity::delete_by_id(sid).exec(db).await {
                    Ok(_) => deleted_count += 1,
                    Err(e) => failed.push(json!({ "id": sid, "error": e.to_string() })),
                }
            }
            Ok(None) => failed.push(json!({
                "id": sid,
                "error": format!("No submission {} found for assignment {}", sid, assignment_id)
            })),
            Err(e) => failed.push(json!({ "id": sid, "error": e.to_string() })),
        }
    }

    let message = format!(
        "Deleted {}/{} submissions",
        deleted_count,
        req.submission_ids.len()
    );

    let data = json!({
        "deleted": deleted_count,
        "failed": failed
    });

    let response = ApiResponse::success(data, message);

    (StatusCode::OK, Json(response))
}
