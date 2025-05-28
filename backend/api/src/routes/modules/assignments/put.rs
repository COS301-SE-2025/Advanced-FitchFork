use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use db::models::assignment::{Assignment, AssignmentType};
use serde::Serialize;
use serde_json::Value;

use crate::response::ApiResponse;
#[derive(Debug, Serialize)]

pub struct AssignmentResponse {
    pub id: i64,
    pub module_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub assignment_type: AssignmentType,
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
            assignment_type: assignment.assignment_type,
            available_from: assignment.available_from,
            due_date: assignment.due_date,
            created_at: assignment.created_at,
            updated_at: assignment.updated_at,
        }
    }
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
    Json(req): Json<Value>,
) -> impl IntoResponse {
    let name = req.get("name").and_then(|v| v.as_str());
    let description = req.get("description").and_then(|v| v.as_str());
    let available_from = req.get("available_from").and_then(|v| v.as_str());
    let due_date = req.get("due_date").and_then(|v| v.as_str());

    fn validate_and_format(date_str: &str) -> Option<String> {
        DateTime::parse_from_rfc3339(date_str)
            .ok()
            .map(|dt| dt.with_timezone(&Utc).to_rfc3339())
    }

    let assignment_type = match req.get("assignment_type").and_then(|v| v.as_str()) {
        Some("Assignment") => db::models::assignment::AssignmentType::Assignment,
        Some("Practical") => db::models::assignment::AssignmentType::Practical,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<AssignmentResponse>::error(
                    "assignment_type must be either 'Assignment' or 'Practical'",
                )),
            );
        }
    };

    if name.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<AssignmentResponse>::error(
                "Assignment Name is expected",
            )),
        );
    }

    let Some(available_from_raw) = available_from else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<AssignmentResponse>::error(
                "Release date is expected",
            )),
        );
    };

    let Some(due_date_raw) = due_date else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<AssignmentResponse>::error(
                "Due date is expected",
            )),
        );
    };

    let available_from = match validate_and_format(available_from_raw) {
        Some(date) => date,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<AssignmentResponse>::error(
                    "Release date must be in valid ISO 8601 format",
                )),
            );
        }
    };

    let due_date = match validate_and_format(due_date_raw) {
        Some(date) => date,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<AssignmentResponse>::error(
                    "Due date must be in valid ISO 8601 format",
                )),
            );
        }
    };

    let name = name.unwrap();

    match db::models::assignment::Assignment::edit(
        Some(db::pool::get()),
        assignment_id,
        module_id,
        name,
        description,
        assignment_type,
        &available_from,
        &due_date,
    )
    .await
    {
        Ok(module) => {
            let res = AssignmentResponse::from(module);
            (
                StatusCode::OK,
                Json(ApiResponse::success(res, "Assignment updated successfully")),
            )
        }
        Err(e) => {
            if e.to_string().contains("no rows") {
                (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::<AssignmentResponse>::error(
                        "Assignment not found",
                    )),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<AssignmentResponse>::error(
                        "Failed to update assignment",
                    )),
                )
            }
        }
    }
}
