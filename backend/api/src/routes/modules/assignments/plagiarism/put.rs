use crate::response::ApiResponse;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use db::models::plagiarism_case::{Column as PlagiarismColumn, Entity as PlagiarismEntity, Status};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};
use serde::{Deserialize, Serialize};
use util::state::AppState;

#[derive(Serialize, Deserialize)]
pub struct UpdatePlagiarismCasePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similarity: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct PlagiarismCaseResponse {
    id: i64,
    assignment_id: i64,
    submission_id_1: i64,
    submission_id_2: i64,
    description: String,
    status: String,
    similarity: f32,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

/// PUT /api/modules/{module_id}/assignments/{assignment_id}/plagiarism/{case_id}
///
/// Updates an existing plagiarism case's description, status, and/or similarity percentage.
/// Accessible only to lecturers and assistant lecturers assigned to the module.
///
/// # Path Parameters
///
/// - `module_id`: The ID of the parent module
/// - `assignment_id`: The ID of the assignment containing the plagiarism case
/// - `case_id`: The ID of the plagiarism case to update
///
/// # Request Body
///
/// Accepts a JSON payload with optional fields (at least one must be provided):
/// - `description` (string): New description for the case
/// - `status` (string): New status ("review", "flagged", or "reviewed")
/// - `similarity` (number): New similarity percentage in **[0.0, 100.0]**
///
/// # Returns
///
/// - `200 OK` with the updated plagiarism case on success
/// - `400 BAD REQUEST` for invalid parameters or missing update fields
/// - `403 FORBIDDEN` if user lacks required permissions
/// - `404 NOT FOUND` if the specified plagiarism case doesn't exist
/// - `500 INTERNAL SERVER ERROR` for database errors or update failures
///
/// # Example Request
///
/// ```json
/// {
///   "description": "Lecturer has reviewed the case and added comments.",
///   "status": "reviewed",
///   "similarity": 68.25
/// }
/// ```
///
/// # Example Response (200 OK)
///
/// ```json
/// {
///   "success": true,
///   "message": "Plagiarism case updated successfully",
///   "data": {
///     "id": 17,
///     "assignment_id": 3,
///     "submission_id_1": 42,
///     "submission_id_2": 51,
///     "description": "Lecturer has reviewed the case and added comments.",
///     "status": "reviewed",
///     "similarity": 68.25,
///     "created_at": "2024-05-20T14:30:00Z",
///     "updated_at": "2024-05-20T15:45:00Z"
///   }
/// }
/// ```
///
/// # Example Responses
///
/// - `400 Bad Request` (missing update fields)
/// ```json
/// {
///   "success": false,
///   "message": "At least one field (description, status, or similarity) must be provided"
/// }
/// ```
///
/// - `400 Bad Request` (invalid status)
/// ```json
/// {
///   "success": false,
///   "message": "Invalid status value. Must be one of: 'review', 'flagged', 'reviewed'"
/// }
/// ```
///
/// - `400 Bad Request` (invalid similarity)
/// ```json
/// {
///   "success": false,
///   "message": "Invalid similarity: must be between 0 and 100"
/// }
/// ```
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
///   "message": "Failed to update plagiarism case"
/// }
/// ```
pub async fn update_plagiarism_case(
    State(app_state): State<AppState>,
    Path((_, assignment_id, case_id)): Path<(i64, i64, i64)>,
    Json(payload): Json<UpdatePlagiarismCasePayload>,
) -> impl IntoResponse {
    // Require at least one updatable field
    if payload.description.is_none() && payload.status.is_none() && payload.similarity.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<PlagiarismCaseResponse>::error(
                "At least one field (description, status, or similarity) must be provided",
            )),
        );
    }

    // Validate similarity, if provided
    if let Some(sim) = payload.similarity {
        if !(0.0..=100.0).contains(&sim) {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<PlagiarismCaseResponse>::error(
                    "Invalid similarity: must be between 0 and 100",
                )),
            );
        }
    }

    // Fetch the case
    let case = match PlagiarismEntity::find_by_id(case_id)
        .filter(PlagiarismColumn::AssignmentId.eq(assignment_id))
        .one(app_state.db())
        .await
    {
        Ok(Some(case)) => case,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<PlagiarismCaseResponse>::error(
                    "Plagiarism case not found",
                )),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<PlagiarismCaseResponse>::error(format!(
                    "Database error: {}",
                    e
                ))),
            );
        }
    };

    // Apply updates
    let mut case = case.into_active_model();

    if let Some(description) = payload.description {
        case.description = sea_orm::ActiveValue::Set(description);
    }

    if let Some(status_str) = payload.status {
        let status = match status_str.as_str() {
            "review" => Status::Review,
            "flagged" => Status::Flagged,
            "reviewed" => Status::Reviewed,
            _ => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<PlagiarismCaseResponse>::error(
                        "Invalid status value. Must be one of: 'review', 'flagged', 'reviewed'",
                    )),
                );
            }
        };
        case.status = sea_orm::ActiveValue::Set(status);
    }

    if let Some(sim) = payload.similarity {
        case.similarity = sea_orm::ActiveValue::Set(sim);
    }

    case.updated_at = sea_orm::ActiveValue::Set(Utc::now());

    // Persist
    let updated_case = match case.update(app_state.db()).await {
        Ok(updated) => updated,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<PlagiarismCaseResponse>::error(format!(
                    "Failed to update plagiarism case: {}",
                    e
                ))),
            );
        }
    };

    // Respond
    let response = PlagiarismCaseResponse {
        id: updated_case.id,
        assignment_id: updated_case.assignment_id,
        submission_id_1: updated_case.submission_id_1,
        submission_id_2: updated_case.submission_id_2,
        description: updated_case.description,
        status: updated_case.status.to_string(),
        similarity: updated_case.similarity, // <- new
        created_at: updated_case.created_at,
        updated_at: updated_case.updated_at,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            response,
            "Plagiarism case updated successfully",
        )),
    )
}
