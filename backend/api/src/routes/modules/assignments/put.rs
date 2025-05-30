use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};

use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

use crate::response::ApiResponse;

use db::{
    connect,
    models::assignment::{self, Model as Assignment, AssignmentType},
};

#[derive(Debug, Serialize)]
pub struct AssignmentResponse {
    pub id: i64,
    pub module_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub assignment_type: String,
    pub available_from: String,
    pub due_date: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Assignment> for AssignmentResponse {
    fn from(assignment: Assignment) -> Self {
        Self {
            id: assignment.id,
            module_id: assignment.module_id,
            name: assignment.name,
            description: assignment.description,
            assignment_type: assignment.assignment_type.to_string(),
            available_from: assignment.available_from.to_rfc3339(),
            due_date: assignment.due_date.to_rfc3339(),
            created_at: assignment.created_at.to_rfc3339(),
            updated_at: assignment.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct EditAssignmentRequest {
    pub name: String,
    pub description: Option<String>,
    pub assignment_type: String,
    pub available_from: String,
    pub due_date: String,
}

/// Edits a specific assignment by its ID and module ID.
///
/// # Arguments
///
/// The arguments are extracted automatically from the HTTP request:
/// - Path parameters `(module_id, assignment_id)` identify the assignment to edit.
/// - JSON body with the following fields:
///   - `name` (string, required): The new name of the assignment.
///   - `description` (string, optional): The new description of the assignment.
///   - `assignment_type` (string, required): The type of the assignment. Must be either `"Assignment"` or `"Practical"`.
///   - `available_from` (string, required): The new availability date (ISO 8601 format).
///   - `due_date` (string, required): The new due date (ISO 8601 format).
///
/// # Returns
///
/// Returns an HTTP response indicating the result:
/// - `200 OK` with the updated assignment data if successful.
/// - `400 BAD REQUEST` if any required fields are missing or malformed (e.g., invalid dates).
/// - `404 NOT FOUND` if no assignment was found matching the given `module_id` and `assignment_id`.
/// - `500 INTERNAL SERVER ERROR` if the update operation fails for other reasons.
pub async fn edit_assignment(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(req): Json<EditAssignmentRequest>,
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