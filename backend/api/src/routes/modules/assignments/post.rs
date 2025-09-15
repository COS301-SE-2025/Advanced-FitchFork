//! Assignment creation route.
//!
//! Provides an endpoint for creating a new assignment in a module:
//! - `POST /api/modules/{module_id}/assignments`
//!
//! Key points:
//! - Assignments are created in the `setup` state by default.
//! - Only lecturers or admins assigned to the module can create assignments.
//! - Dates must be in ISO 8601 format (RFC 3339).
//! - `assignment_type` must be either `"assignment"` or `"practical"`.
//!
//! Responses include standard `200 OK`, `400 Bad Request` for validation errors, and `500 Internal Server Error` for database issues.

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use crate::response::ApiResponse;
use crate::routes::modules::assignments::common::{AssignmentRequest, AssignmentResponse};
use services::service::Service;
use services::assignment::{AssignmentService, AssignmentType, CreateAssignment};

/// POST /api/modules/{module_id}/assignments
///
/// Create a new assignment in a module.  
/// The assignment is always created in the `setup` state by default.  
/// Only accessible by lecturers or admins assigned to the module.
///
/// ### Path Parameters
/// - `module_id` (`i64`): The ID of the module to create the assignment in.
///
/// ### Request Body (JSON)
/// - `name` (`string`, required): The name of the assignment.
/// - `description` (`string`, optional): A description of the assignment.
/// - `assignment_type` (`string`, required): The type of assignment. Must be either `"assignment"` or `"practical"`.
/// - `available_from` (`string`, required): The date/time from which the assignment is available (ISO 8601 format).
/// - `due_date` (`string`, required): The due date/time for the assignment (ISO 8601 format).
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "message": "Assignment created successfully",
///   "data": {
///     "id": 123,
///     "module_id": 456,
///     "name": "Assignment 1",
///     "description": "This is a sample assignment",
///     "assignment_type": "Assignment",
///     "available_from": "2024-01-01T00:00:00Z",
///     "due_date": "2024-01-31T23:59:59Z",
///     "created_at": "2024-01-01T00:00:00Z",
///     "updated_at": "2024-01-01T00:00:00Z"
///   }
/// }
/// ```
///
/// - `400 Bad Request`
/// ```json
/// {
///   "success": false,
///   "message": "Invalid available_from datetime" // or "Invalid due_date datetime" or "assignment_type must be 'assignment' or 'practical'"
/// }
/// ```
///
/// - `500 Internal Server Error`
/// ```json
/// {
///   "success": false,
///   "message": "Assignment could not be inserted" // or "Database error"
/// }
/// ```
pub async fn create_assignment(
    Path(module_id): Path<i64>,
    Json(req): Json<AssignmentRequest>,
) -> impl IntoResponse {
    let available_from = match DateTime::parse_from_rfc3339(&req.available_from)
        .map(|dt| dt.with_timezone(&Utc)) {
        Ok(date) => date,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<AssignmentResponse>::error(
                    "Invalid available_from datetime",
                )),
            );
        }
    };

    let due_date = match DateTime::parse_from_rfc3339(&req.due_date)
        .map(|dt| dt.with_timezone(&Utc)) {
        Ok(date) => date,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<AssignmentResponse>::error(
                    "Invalid due_date datetime",
                )),
            );
        }
    };

    let assignment_type = match req.assignment_type.parse::<AssignmentType>() {
        Ok(assignment_type) => assignment_type,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<AssignmentResponse>::error("assignment_type must be 'assignment' or 'practical'")),
            );
        }
    };

    match AssignmentService::create(
        CreateAssignment {
            id: None,
            module_id,
            name: req.name,
            description: req.description,
            assignment_type,
            available_from,
            due_date,
        }
    ).await {
        Ok(model) => {
            let response = AssignmentResponse::from(model);
            (
                StatusCode::CREATED,
                Json(ApiResponse::success(
                    response,
                    "Assignment created successfully",
                )),
            )
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<AssignmentResponse>::error(
                    format!("Database error: {}", e),
                )),
            )
        }
    }
}