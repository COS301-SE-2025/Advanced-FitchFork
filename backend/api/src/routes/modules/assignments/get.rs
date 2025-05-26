use axum::extract::Query;
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
}

#[derive(Debug, Deserialize)]
pub struct FilterReq {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub sort: Option<String>,
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
/// - `name`: (Optional) A filter to match assignments by name.
/// - `assignment_type`: (Optional) A filter to match assignments by their type.
/// - `available_before`: (Optional) Filter assignments that become available before this date/time.
/// - `available_after`: (Optional) Filter assignments that become available after this date/time.
/// - `due_before`: (Optional) Filter assignments that are due before this date/time.
/// - `due_after`: (Optional) Filter assignments that are due after this date/time.
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
        if !valid_fields.contains(&params.sort.as_ref().unwrap().as_str()) {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<FilterResponse>::error("Invalid field used")),
            );
        }
    }


    let res = Assignment::filter(
        Some(pool::get()),
        page,
        length,
        params.sort,
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
