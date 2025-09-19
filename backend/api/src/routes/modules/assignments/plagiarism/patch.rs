use crate::response::ApiResponse;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use db::models::plagiarism_case::{Column as PlagiarismColumn, Entity as PlagiarismEntity, Status};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};
use serde::Serialize;
use util::state::AppState;

#[derive(Debug, Serialize)]
pub struct FlaggedCaseResponse {
    id: i64,
    status: String,
    updated_at: DateTime<Utc>,
}

/// PATCH /api/modules/{module_id}/assignments/{assignment_id}/plagiarism/{case_id}/flag
///
/// Flags a plagiarism case after manual review, indicating confirmed plagiarism.
/// Accessible only to lecturers and assistant lecturers assigned to the module.
///
/// # Path Parameters
///
/// - `module_id`: The ID of the parent module
/// - `assignment_id`: The ID of the assignment containing the plagiarism case
/// - `case_id`: The ID of the plagiarism case to flag
///
/// # Returns
///
/// Returns an HTTP response indicating the result:
/// - `200 OK` with minimal case information on success
/// - `403 FORBIDDEN` if user lacks required permissions
/// - `404 NOT FOUND` if specified plagiarism case doesn't exist
/// - `500 INTERNAL SERVER ERROR` for database errors or update failures
///
/// The response body includes only essential fields after the status change.
///
/// # Example Response (200 OK)
///
/// ```json
/// {
///   "success": true,
///   "message": "Plagiarism case flagged",
///   "data": {
///     "id": 17,
///     "status": "flagged",
///     "updated_at": "2024-05-20T16:30:00Z"
///   }
/// }
/// ```
///
/// # Example Responses
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "message": "Plagiarism case not found"
/// }
/// ```
///
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "message": "Failed to update plagiarism case: [error details]"
/// }
/// ```
///
/// # Notes
///
/// - This operation updates the case status to "flagged" and sets the current timestamp to `updated_at`
/// - Only users with lecturer or assistant lecturer roles assigned to the module can perform this action
/// - Considered an irreversible action indicating confirmed plagiarism
pub async fn patch_plagiarism_flag(
    State(app_state): State<AppState>,
    Path((_, assignment_id, case_id)): Path<(i64, i64, i64)>,
) -> impl IntoResponse {
    let case = match PlagiarismEntity::find()
        .filter(PlagiarismColumn::Id.eq(case_id))
        .filter(PlagiarismColumn::AssignmentId.eq(assignment_id))
        .one(app_state.db())
        .await
    {
        Ok(Some(case)) => case,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("Plagiarism case not found")),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!("Database error: {}", e))),
            );
        }
    };

    let mut active_case = case.into_active_model();
    active_case.status = Set(Status::Flagged);
    active_case.updated_at = Set(Utc::now());

    let updated_case = match active_case.update(app_state.db()).await {
        Ok(case) => case,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!(
                    "Failed to update plagiarism case: {}",
                    e
                ))),
            );
        }
    };

    let response_data = FlaggedCaseResponse {
        id: updated_case.id as i64,
        status: updated_case.status.to_string(),
        updated_at: updated_case.updated_at,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            response_data,
            "Plagiarism case flagged",
        )),
    )
}

#[derive(Debug, Serialize)]
pub struct ReviewedCaseResponse {
    id: i64,
    status: String,
    updated_at: DateTime<Utc>,
}

/// PATCH /api/modules/{module_id}/assignments/{assignment_id}/plagiarism/{case_id}/review
///
/// Marks a plagiarism case as reviewed after manual inspection, indicating it's been cleared of plagiarism concerns.
/// Accessible only to lecturers and assistant lecturers assigned to the module.
///
/// # Path Parameters
///
/// - `module_id`: The ID of the parent module
/// - `assignment_id`: The ID of the assignment containing the plagiarism case
/// - `case_id`: The ID of the plagiarism case to mark as reviewed
///
/// # Returns
///
/// Returns an HTTP response indicating the result:
/// - `200 OK` with minimal case information on success
/// - `403 FORBIDDEN` if user lacks required permissions
/// - `404 NOT FOUND` if specified plagiarism case doesn't exist
/// - `500 INTERNAL SERVER ERROR` for database errors or update failures
///
/// The response body includes only essential fields after the status change.
///
/// # Example Response (200 OK)
///
/// ```json
/// {
///   "success": true,
///   "message": "Plagiarism case marked as reviewed",
///   "data": {
///     "id": 17,
///     "status": "reviewed",
///     "updated_at": "2024-05-20T17:45:00Z"
///   }
/// }
/// ```
///
/// # Example Responses
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "message": "Plagiarism case not found"
/// }
/// ```
///
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "message": "Failed to update plagiarism case: [error details]"
/// }
/// ```
///
/// # Notes
///
/// - This operation updates the case status to "reviewed" and sets the current timestamp to `updated_at`
/// - Only users with lecturer or assistant lecturer roles assigned to the module can perform this action
/// - Typically indicates the case was investigated and determined not to be plagiarism
pub async fn patch_plagiarism_review(
    State(app_state): State<AppState>,
    Path((_, assignment_id, case_id)): Path<(i64, i64, i64)>,
) -> impl IntoResponse {
    let case = match PlagiarismEntity::find()
        .filter(PlagiarismColumn::Id.eq(case_id))
        .filter(PlagiarismColumn::AssignmentId.eq(assignment_id))
        .one(app_state.db())
        .await
    {
        Ok(Some(case)) => case,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("Plagiarism case not found")),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!("Database error: {}", e))),
            );
        }
    };

    let mut active_case = case.into_active_model();
    active_case.status = Set(Status::Reviewed);
    active_case.updated_at = Set(Utc::now());

    let updated_case = match active_case.update(app_state.db()).await {
        Ok(case) => case,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!(
                    "Failed to update plagiarism case: {}",
                    e
                ))),
            );
        }
    };

    let response_data = ReviewedCaseResponse {
        id: updated_case.id as i64,
        status: updated_case.status.to_string(),
        updated_at: updated_case.updated_at,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            response_data,
            "Plagiarism case marked as reviewed",
        )),
    )
}
