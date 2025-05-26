use std::path::PathBuf;
use axum::response::{Json, Response};
use axum::{extract::Path, http::StatusCode, response::IntoResponse};
use axum::http::{header, HeaderMap, HeaderValue};
use db::{
    models::assignment::{Assignment, AssignmentType},
    models::assignment_files::AssignmentFiles,
    pool,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::auth::AuthUser;
use crate::response::ApiResponse;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

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
    pub files: Vec<File2>,
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
pub struct File2 {
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

                    let converted_files: Vec<File2> = files
                        .into_iter()
                        .map(|f| File2 {
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




/// Represents a file associated with an assignment.
///
/// This struct is used in both upload and download operations, allowing
/// file metadata to be returned in a consistent API structure.
///
/// - `mime_type` allows distinguishing between file formats.
/// - Timestamps are ISO 8601 formatted.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AssignmentFile {
    pub id: i64,
    pub assignment_id: i64,
    pub filename: String,
    pub path: String,
    pub mime_type: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
/// GET /api/modules/:module_id/assignments/:assignment_id/files/:file_id
///
/// Download a file from an assignment. Accessible to admins or users assigned to the module.
///
/// ### Response
/// - `200 OK`: Returns file with correct headers
/// - `403 Forbidden`: User not authorized
/// - `404 Not Found`: File or assignment not found
pub async fn download_file(Path((module_id, assignment_id, file_id)): Path<(i64, i64, i64)>, AuthUser(claims): AuthUser, ) -> Response {
    let pool = pool::get();

    let is_authorized = claims.admin || {
        let query = r#"
            SELECT EXISTS(
                SELECT 1 FROM module_lecturers WHERE module_id = ? AND user_id = ?
                UNION
                SELECT 1 FROM module_tutors WHERE module_id = ? AND user_id = ?
                UNION
                SELECT 1 FROM module_students WHERE module_id = ? AND user_id = ?
            )
        "#;

        sqlx::query_scalar(query).bind(module_id).bind(claims.sub).bind(module_id).bind(claims.sub).bind(module_id).bind(claims.sub).fetch_one(pool).await.unwrap_or(false)
    };

    if !is_authorized {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<AssignmentFile>::error("You do not have permission to access this file")),
        ).into_response();
    }

    let file: Option<AssignmentFile> = sqlx::query_as::<_, AssignmentFile>(
        "SELECT * FROM assignment_files WHERE id = ? AND assignment_id = ?"
    ).bind(file_id).bind(assignment_id).fetch_optional(pool).await.unwrap_or(None);

    let Some(file) = file else {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<AssignmentFile>::error("File not found")),
        ).into_response();
    };

    let fs_path = PathBuf::from(&file.path);
    if !fs_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<AssignmentFile>::error("File missing on disk")),
        ).into_response();
    }

    match File::open(&fs_path).await {
        Ok(mut f) => {
            let mut buf = Vec::new();
            if let Err(_) = f.read_to_end(&mut buf).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<AssignmentFile>::error("Failed to read file")),
                ).into_response();
            }

            let mut headers = HeaderMap::new();

            let disposition = HeaderValue::from_str(&format!("attachment; filename=\"{}\"", file.filename))
                .unwrap_or_else(|_| HeaderValue::from_static("attachment"));

            headers.insert(header::CONTENT_DISPOSITION, disposition);
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));

            (StatusCode::OK, headers, buf).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignmentFile>::error("Could not open file")),
        ).into_response(),
    }
}