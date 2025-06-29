use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};

use chrono::{DateTime, Utc};

use crate::response::ApiResponse;

use db::{
    connect,
    models::assignment::{self, AssignmentType},
};
use crate::routes::modules::assignments::common::{AssignmentRequest, AssignmentResponse};

/// PUT /api/modules/:module_id/assignments/:assignment_id
///
/// Edit an existing assignment in a module. Only accessible by lecturers or admins assigned to the module.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to edit
///
/// ### Request Body (JSON)
/// - `name` (string, required): The new name of the assignment
/// - `description` (string, optional): The new description of the assignment
/// - `assignment_type` (string, required): The type of assignment. Must be either "assignment" or "practical"
/// - `available_from` (string, required): The new date/time from which the assignment is available (ISO 8601 format)
/// - `due_date` (string, required): The new due date/time for the assignment (ISO 8601 format)
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "message": "Assignment updated successfully",
///   "data": {
///     "id": 123,
///     "module_id": 456,
///     "name": "Updated Assignment Name",
///     "description": "Updated assignment description",
///     "assignment_type": "Assignment",
///     "available_from": "2024-01-01T00:00:00Z",
///     "due_date": "2024-01-31T23:59:59Z",
///     "created_at": "2024-01-01T00:00:00Z",
///     "updated_at": "2024-01-15T12:30:00Z"
///   }
/// }
/// ```
///
/// - `400 Bad Request`
/// ```json
/// {
///   "success": false,
///   "message": "Invalid available_from datetime format" // or "Invalid due_date datetime format" or "assignment_type must be 'assignment' or 'practical'"
/// }
/// ```
///
/// - `404 Not Found`
/// ```json
/// {
///   "success": false,
///   "message": "Assignment not found"
/// }
/// ```
///
/// - `500 Internal Server Error`
/// ```json
/// {
///   "success": false,
///   "message": "Failed to update assignment"
/// }
/// ```
///
pub async fn edit_assignment(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(req): Json<AssignmentRequest>,
) -> impl IntoResponse {
    let db = connect().await;

    let available_from = match DateTime::parse_from_rfc3339(&req.available_from)
        .map(|dt| dt.with_timezone(&Utc))
    {
        Ok(dt) => dt,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<AssignmentResponse>::error("Invalid available_from datetime format")),
            );
        }
    };

    let due_date = match DateTime::parse_from_rfc3339(&req.due_date)
        .map(|dt| dt.with_timezone(&Utc))
    {
        Ok(dt) => dt,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<AssignmentResponse>::error("Invalid due_date datetime format")),
            );
        }
    };

    let assignment_type = match req.assignment_type.parse::<AssignmentType>() {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<AssignmentResponse>::error(
                    "assignment_type must be 'assignment' or 'practical'",
                )),
            );
        }
    };

    match assignment::Model::edit(
        &db,
        assignment_id,
        module_id,
        &req.name,
        req.description.as_deref(),
        assignment_type, // pass enum here
        available_from,
        due_date,
    )
    .await
    {
        Ok(updated) => {
            let res = AssignmentResponse::from(updated);
            (
                StatusCode::OK,
                Json(ApiResponse::success(res, "Assignment updated successfully")),
            )
        }
        Err(sea_orm::DbErr::RecordNotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<AssignmentResponse>::error("Assignment not found")),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignmentResponse>::error("Failed to update assignment")),
        ),
    }
}