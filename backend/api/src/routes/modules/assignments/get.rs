use std::path::PathBuf;
use axum::response::{Json, Response};
use axum::extract::Query;
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
use tokio::fs::File as FsFile;
use tokio::io::AsyncReadExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct AssignmentResponse {
    pub assignment: AssignmentDetailResponse,
    pub files: Vec<File>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AssignmentDetailResponse {
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

impl From<Assignment> for AssignmentDetailResponse {
    fn from(assignment: Assignment) -> Self {
        AssignmentDetailResponse {
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

impl From<Assignment> for AssignmentResponse {
    fn from(assignment: Assignment) -> Self {
        Self {
            assignment: AssignmentDetailResponse {
                id: assignment.id,
                module_id: assignment.module_id,
                name: assignment.name,
                description: assignment.description,
                assignment_type: assignment.assignment_type,
                available_from: assignment.available_from,
                due_date: assignment.due_date,
                created_at: assignment.created_at,
                updated_at: assignment.updated_at,
            },
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

/// Retrieves a specific assignment along with its associated files.
///
/// # Arguments
///
/// The arguments are extracted automatically from the HTTP route:
/// - Path parameters `(module_id, assignment_id)`: Identify the assignment within the specified module.
///
/// # Returns
///
/// Returns an HTTP response indicating the result:
/// - `200 OK` with the assignment details and associated files if found.
/// - `404 NOT FOUND` if no assignment is found with the given `module_id` and `assignment_id`.
/// - `500 INTERNAL SERVER ERROR` if a database error occurs or if the associated files cannot be retrieved.
///
/// The response body is a JSON object using a standardized API response format and includes:
/// - Assignment details.
/// - A list of associated files (each represented as a `File` object).

pub async fn get_assignment(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let assignment_res = Assignment::get_by_id(Some(pool::get()), assignment_id, module_id).await;

    match assignment_res {
        Ok(Some(assignment)) => {
            let files_res =
                AssignmentFiles::get_by_assignment_id(Some(pool::get()), assignment_id).await;

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
                        Json(ApiResponse::success(
                            response,
                            "Assignment retrieved successfully",
                        )),
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
} // test




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
                SELECT 1 FROM module_tutors    WHERE module_id = ? AND user_id = ?
                UNION
                SELECT 1 FROM module_students  WHERE module_id = ? AND user_id = ?
            )
        "#;
        sqlx::query_scalar(query)
            .bind(module_id).bind(claims.sub)
            .bind(module_id).bind(claims.sub)
            .bind(module_id).bind(claims.sub)
            .fetch_one(pool)
            .await
            .unwrap_or(false)
    };

    if !is_authorized {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<AssignmentFile>::error("You do not have permission to access this file")),
        )
            .into_response();
    }

    let file_res = sqlx::query_as::<_, AssignmentFile>(
        r#"
        SELECT id, assignment_id, filename, path
          FROM assignment_files
         WHERE id = ? AND assignment_id = ?
        "#,
    )
        .bind(file_id)
        .bind(assignment_id)
        .fetch_optional(pool)
        .await;

    let file = match file_res {
        Ok(Some(f)) => f,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<AssignmentFile>::error("File not found")),
            )
                .into_response();
        }
        Err(err) => {
            eprintln!("DB error fetching file: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<AssignmentFile>::error("Database error")),
            )
                .into_response();
        }
    };

    let fs_path = PathBuf::from(&file.path);
    if !fs_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<AssignmentFile>::error("File missing on disk")),
        )
            .into_response();
    }
    
    let mut f = match FsFile::open(&fs_path).await {
        Ok(f) => f,
        Err(err) => {
            eprintln!("File open error: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<AssignmentFile>::error("Could not open file")),
            )
                .into_response();
        }
    };
    let mut buf = Vec::new();
    if let Err(err) = f.read_to_end(&mut buf).await {
        eprintln!("File read error: {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignmentFile>::error("Failed to read file")),
        )
            .into_response();
    }

    
    let mut headers = HeaderMap::new();
    let disposition = HeaderValue::from_str(&format!("attachment; filename=\"{}\"", file.filename))
        .unwrap_or_else(|_| HeaderValue::from_static("attachment"));
    headers.insert(header::CONTENT_DISPOSITION, disposition);
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));

    (StatusCode::OK, headers, buf).into_response()
}


pub async fn list_files(Path((module_id, assignment_id)): Path<(i64, i64)>, AuthUser(claims): AuthUser, ) -> Response {
    match Assignment::get_by_id(Some(pool::get()), assignment_id, module_id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<Vec<File>>::error(
                    "Assignment not found".to_string(),
                )),
            )
                .into_response();
        }
        Err(err) => {
            eprintln!("DB error checking assignment: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<File>>::error(
                    "Database error".to_string(),
                )),
            ).into_response();
        }
    }

    let is_authorized = claims.admin
        || {
        let q = r#"
                SELECT EXISTS(
                  SELECT 1 FROM module_lecturers WHERE module_id = ? AND user_id = ?
                  UNION
                  SELECT 1 FROM module_tutors    WHERE module_id = ? AND user_id = ?
                  UNION
                  SELECT 1 FROM module_students  WHERE module_id = ? AND user_id = ?
                )
            "#;
        sqlx::query_scalar(q).bind(module_id).bind(claims.sub).bind(module_id).bind(claims.sub).bind(module_id).bind(claims.sub).fetch_one(pool::get()).await.unwrap_or(false)
    };
    if !is_authorized {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<Vec<File>>::error(
                "You do not have permission to view files for this assignment".to_string(),
            )),
        ).into_response();
    }

    match AssignmentFiles::get_by_assignment_id(Some(pool::get()), assignment_id).await {
        Ok(files) => {
            let file_list: Vec<File> = files
                .into_iter()
                .map(|f| File {
                    id: f.id.to_string(),
                    filename: f.filename,
                    path: f.path,
                    created_at: f.created_at,
                    updated_at: f.updated_at,
                })
                .collect();

            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    file_list,
                    "Assignment files retrieved successfully",
                )),
            ).into_response()
        }
        Err(err) => {
            eprintln!("DB error fetching files: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<File>>::error(
                    "Failed to retrieve files".to_string(),
                )),
            ).into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FilterReq {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub sort: Option<String>,
    pub query: Option<String>,
    pub name: Option<String>,
    pub assignment_type: Option<String>,
    pub available_before: Option<String>,
    pub available_after: Option<String>,
    pub due_before: Option<String>,
    pub due_after: Option<String>,
}
#[derive(Serialize)]
pub struct FilterResponse {
    pub assignments: Vec<AssignmentDetailResponse>,
    pub page: i32,
    pub per_page: i32,
    pub total: i32,
}
impl From<(Vec<Assignment>, i32, i32, i32)> for FilterResponse {
    fn from(data: (Vec<Assignment>, i32, i32, i32)) -> Self {
        let (assignments, page, per_page, total) = data;
        Self {
            assignments: assignments
                .into_iter()
                .map(AssignmentDetailResponse::from)
                .collect(),
            page,
            per_page,
            total,
        }
    }
}

/// Retrieves a paginated and optionally filtered list of assignments.
///
/// # Arguments
///
/// The arguments are automatically extracted from query parameters via the `FilterReq` struct:
/// - `page`: (Optional) The page number for pagination. Defaults to 1 if not provided. Minimum value is 1.
/// - `per_page`: (Optional) The number of items per page. Defaults to 20. Maximum is 100. Minimum is 1.
/// - `sort`: (Optional) A comma-separated list of fields to sort by. Prefix with `-` for descending order (e.g., `-due_date`).
/// - `query`: (Optional) A case-insensitive substring match applied to both `name` and `description`.
/// - `name`: (Optional) A case-insensitive filter to match assignment names.
/// - `assignment_type`: (Optional) A filter to match assignments by their type ("Assignment" or "Practical").
/// - `available_before`: (Optional) Filter assignments that become available before this date/time (ISO 8601).
/// - `available_after`: (Optional) Filter assignments that become available after this date/time (ISO 8601).
/// - `due_before`: (Optional) Filter assignments that are due before this date/time (ISO 8601).
/// - `due_after`: (Optional) Filter assignments that are due after this date/time (ISO 8601).
///
/// Allowed sort fields: `"name"`, `"due_date"`, `"available_from"`, `"assignment_type"`, `"created_at"`, `"updated_at"`.
///
/// # Returns
///
/// Returns an HTTP response indicating the result:
/// - `200 OK` with a list of matching assignments, paginated and wrapped in a standardized response format.
/// - `400 BAD REQUEST` if an invalid field is used for sorting.
/// - `500 INTERNAL SERVER ERROR` if a database error occurs while retrieving the assignments.
///
/// The response body contains:
/// - A paginated list of assignments.
/// - Metadata: current page, items per page, and total items.

pub async fn get_assignments(Query(params): Query<FilterReq>) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let length = params.per_page.unwrap_or(20).min(100).max(1);
    if params.sort.is_some() {
        let valid_fields = [
            "name",
            "due_date",
            "available_from",
            "assignment_type",
            "created_at",
            "updated_at",
        ];
        if let Some(sort_field) = &params.sort {
            let field = sort_field.trim_start_matches('-');
            if !valid_fields.contains(&field) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<FilterResponse>::error("Invalid field used")),
                );
            }
        }
    }

    let res = Assignment::filter(
        Some(pool::get()),
        page,
        length,
        params.sort,
        params.query,
        params.name,
        params.assignment_type,
        params.available_before,
        params.available_after,
        params.due_before,
        params.due_after,
    )
    .await;

    match res {
        Ok(data) => {
            let total = data.len() as i32;
            let response: FilterResponse = FilterResponse::from((data, page, length, total));
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "Modules retrieved successfully",
                )),
            )
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<FilterResponse>::error(
                "An error occurred while retrieving modules",
            )),
        ),
    }
}
