/// Handles the creation of a new assignment.
///
/// # Request Body
/// Expects a JSON object with the following fields:
/// - `module_id` (i64): The ID of the module to which the assignment belongs. **Required**
/// - `name` (string): The name of the assignment. **Required**
/// - `description` (string): A description of the assignment. *(Optional)*
/// - `available_from` (string): The date from which the assignment is available (ISO 8601 format). **Required**
/// - `due_date` (string): The due date for the assignment (ISO 8601 format). **Required**
/// - `assignment_type` (string): The type of assignment, either `"A"` for Assignment or any other value for Practical. *(Optional, defaults to Practical)*
///
/// # Responses
/// Returns a response indicating the result of the creation operation.
///
/// # Errors
/// Returns an error response if any of the required fields are missing or invalid.
/// Handles the deletion of an assignment.
use axum::{http::StatusCode, response::IntoResponse, Json};
use db::{
    models::assignment::{Assignment, AssignmentType},
    pool,
};
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

pub async fn create(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    let module_id = req.get("module_id").and_then(|v| v.as_i64());
    let name = req.get("name").and_then(|v| v.as_str());
    let description = req.get("description").and_then(|v| v.as_str());
    let available_from = req.get("available_from").and_then(|v| v.as_str());
    let due_date = req.get("due_date").and_then(|v| v.as_str());
    let assignment_type = match req.get("assignment_type").and_then(|v| v.as_str()) {
        Some("A") => db::models::assignment::AssignmentType::Assignment,
        _ => db::models::assignment::AssignmentType::Practical,
    };

    if module_id.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<AssignmentResponse>::error(
                "Module Id is expected",
            )),
        );
    }

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

    let module_id = module_id.unwrap();
    let name = name.unwrap();
    let available_from = available_from.unwrap();
    let due_date = due_date.unwrap();

    let result: Result<Assignment, sqlx::Error> = Assignment::create(
        Some(pool::get()),
        module_id,
        name,
        description,
        assignment_type,
        available_from,
        due_date,
    )
    .await;
    match result {
        Ok(assignment) => {
            let res = AssignmentResponse::from(assignment);
            (
                StatusCode::OK,
                Json(ApiResponse::success(res, "Assignment successfully created")),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignmentResponse>::error(format!(
                "Database error: {}",
                e
            ))),
        ),
    }
}

#[cfg(test)]
mod tests {
    use axum::{http::StatusCode, response::IntoResponse, Json};
    use db::{create_test_db, delete_database};
    use serde_json::json;

    #[tokio::test]
    async fn create_ass() {
        let pool = create_test_db(Some("assignment.db")).await;
        let mut request_json= Json(json!({}));
        let response = crate::routes::assignments::post::create(request_json)
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        request_json = Json(json!({
            "module_id":313312,
            "name": "prac3",
            "available_from": "2025-12-34",
            "due_date": "12-12-1"
        }));
        let response = crate::routes::assignments::post::create(request_json)
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);

        pool.close().await;
        delete_database("test_update_not_found.db");
    }
}
