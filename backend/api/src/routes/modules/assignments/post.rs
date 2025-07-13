use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use chrono::{DateTime, Utc};

use sea_orm::DbErr;

use crate::response::ApiResponse;
use db::{
    connect,
    models::{
        assignment::{
            AssignmentType,
            Model as AssignmentModel,
        }
    },
};
use crate::routes::modules::assignments::common::{AssignmentRequest, AssignmentResponse};

/// POST /api/modules/:module_id/assignments
///
/// Create a new assignment in a module. Only accessible by lecturers or admins assigned to the module.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module to create the assignment in
///
/// ### Request Body (JSON)
/// - `name` (string, required): The name of the assignment
/// - `description` (string, optional): A description of the assignment
/// - `assignment_type` (string, required): The type of assignment. Must be either "assignment" or "practical"
/// - `available_from` (string, required): The date/time from which the assignment is available (ISO 8601 format)
/// - `due_date` (string, required): The due date/time for the assignment (ISO 8601 format)
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
///
pub async fn create(
    Path(module_id): Path<i64>,
    Json(req): Json<AssignmentRequest>,
) -> impl IntoResponse {
    let db = connect().await;

    let available_from =
        match DateTime::parse_from_rfc3339(&req.available_from).map(|dt| dt.with_timezone(&Utc)) {
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

    let due_date =
        match DateTime::parse_from_rfc3339(&req.due_date).map(|dt| dt.with_timezone(&Utc)) {
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

    match AssignmentModel::create(
        &db,
        module_id,
        &req.name,
        req.description.as_deref(),
        assignment_type,
        available_from,
        due_date,
    )
    .await
    {
        Ok(model) => {
            let response = AssignmentResponse::from(model);
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "Assignment created successfully",
                )),
            )
        }
        Err(DbErr::RecordNotInserted) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignmentResponse>::error(
                "Assignment could not be inserted",
            )),
        ),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<AssignmentResponse>::error("Database error")),
            )
        }
    }
}