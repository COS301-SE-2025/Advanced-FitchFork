use axum::response::Json;
use axum::{extract::Path, http::StatusCode, response::IntoResponse};
use db::{
    models::assignment::{Assignment, AssignmentType},
    models::assignment_files::AssignmentFiles,
    pool,
};
use serde::{Deserialize, Serialize};

use crate::response::ApiResponse;

#[derive(Debug, Serialize, Deserialize)]
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
    pub files: Vec<File>,
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
            files: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub id: String,
    pub filename: String,
    pub path: String,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn get_assignment(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let assignment_res = Assignment::get_by_id(Some(pool::get()), assignment_id, module_id).await;

    match assignment_res {
        Ok(Some(assignment)) => {
            let files_res = AssignmentFiles::get_by_assignment_id(Some(pool::get()), assignment_id).await;

            match files_res {
                Ok(files) => {

                    let converted_files: Vec<File> = files
                        .into_iter()
                        .map(|f| File {
                            id: f.id.to_string(), 
                            filename: f.filename,
                            path: f.path,
                            created_at: f.created_at,
                            updated_at: f.updated_at,
                        })
                        .collect();

                    let mut response = AssignmentResponse::from(assignment);
                    response.files = converted_files;

                    (
                        StatusCode::OK,
                        Json(ApiResponse::success(response, "Assignment retrieved successfully")),
                    )
                }
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<AssignmentResponse>::error(
                        "Failed to retrieve files".to_string(),
                    )),
                ),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<AssignmentResponse>::error(
                "Assignment not found".to_string(),
            )),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignmentResponse>::error(
                "An error occurred in the database".to_string(),
            )),
        ),
    }
}
