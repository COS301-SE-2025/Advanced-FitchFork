use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use db::models::assignment::{Assignment, AssignmentType};
use serde::Serialize;

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

pub async fn edit_assignment(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let name = req.get("name").and_then(|v| v.as_str());
    let description = req.get("description").and_then(|v| v.as_str());
    let available_from = req.get("available_from").and_then(|v| v.as_str());
    let due_date = req.get("due_date").and_then(|v| v.as_str());
    let assignment_type = match req.get("assignment_type").and_then(|v| v.as_str()) {
        Some("A") => db::models::assignment::AssignmentType::Assignment,
        _ => db::models::assignment::AssignmentType::Practical,
    };
    if name.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<AssignmentResponse>::error(
                "Assignment Name is expected",
            )),
        );
    }

    if available_from.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<AssignmentResponse>::error(
                "Release date is expected",
            )),
        );
    }

    if due_date.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<AssignmentResponse>::error(
                "Due date is expected",
            )),
        );
    }

    let name = name.unwrap();
    let available_from = available_from.unwrap();
    let due_date = due_date.unwrap();

    match db::models::assignment::Assignment::edit(
        Some(db::pool::get()),
        assignment_id,
        module_id,
        name,
        description,
        assignment_type,
        available_from,
        due_date,
    )
    .await
    {
        Ok(module) => {
            let res = AssignmentResponse::from(module);
            return (
                StatusCode::OK,
                Json(ApiResponse::success(res, "Module updated successfully")),
            );
        }
        Err(e) => {
            if e.to_string().contains("no rows") {
                return (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::<AssignmentResponse>::error("Assignment not found")),
                );
            }
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<AssignmentResponse>::error(&format!(
                    "Failed to update module:",
                ))),
            );
        }
    }
}
