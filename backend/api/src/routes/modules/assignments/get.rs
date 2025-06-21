use std::{env, path::PathBuf};

use axum::{
    extract::{Extension, Path, Query},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
};

use chrono::{DateTime, Utc};
use db::models::AssignmentTask;
use db::models::assignment_task::{ActiveModel, Entity};
use serde::{Deserialize, Serialize};

use tokio::{fs::File as FsFile, io::AsyncReadExt};

use sea_orm::{
    sea_query::Expr, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder,
};

use crate::auth::AuthUser;
use crate::response::ApiResponse;
use db::{
    connect,
    models::{
        assignment::{
            self, AssignmentType, Column as AssignmentColumn, Entity as AssignmentEntity,
            Model as AssignmentModel,
        }, assignment_file::{self, Column as FileColumn, Entity as FileEntity}, assignment_submission, assignment_task::Column, user, User
    },
};

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
    pub assignment_type: String,
    pub available_from: String,
    pub due_date: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<assignment::Model> for AssignmentDetailResponse {
    fn from(a: assignment::Model) -> Self {
        AssignmentDetailResponse {
            id: a.id,
            module_id: a.module_id as i64,
            name: a.name,
            description: a.description,
            assignment_type: a.assignment_type.to_string(),
            available_from: a.available_from.to_rfc3339(),
            due_date: a.due_date.to_rfc3339(),
            created_at: a.created_at.to_rfc3339(),
            updated_at: a.updated_at.to_rfc3339(),
        }
    }
}

impl From<AssignmentModel> for AssignmentResponse {
    fn from(assignment: AssignmentModel) -> Self {
        Self {
            assignment: AssignmentDetailResponse {
                id: assignment.id,
                module_id: assignment.module_id as i64,
                name: assignment.name,
                description: assignment.description,
                assignment_type: assignment.assignment_type.to_string(),
                available_from: assignment.available_from.to_rfc3339(),
                due_date: assignment.due_date.to_rfc3339(),
                created_at: assignment.created_at.to_rfc3339(),
                updated_at: assignment.updated_at.to_rfc3339(),
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
    let db = connect().await;

    let assignment_res = assignment::Entity::find()
        .filter(assignment::Column::Id.eq(assignment_id as i32))
        .filter(assignment::Column::ModuleId.eq(module_id as i32))
        .one(&db)
        .await;

    match assignment_res {
        Ok(Some(a)) => {
            let files_res = assignment_file::Entity::find()
                .filter(assignment_file::Column::AssignmentId.eq(a.id))
                .all(&db)
                .await;

            match files_res {
                Ok(files) => {
                    let converted_files: Vec<File> = files
                        .into_iter()
                        .map(|f| File {
                            id: f.id.to_string(),
                            filename: f.filename,
                            path: f.path,
                            created_at: f.created_at.to_rfc3339(),
                            updated_at: f.updated_at.to_rfc3339(),
                        })
                        .collect();

                    let response = AssignmentResponse {
                        assignment: AssignmentDetailResponse::from(a),
                        files: converted_files,
                    };

                    (
                        StatusCode::OK,
                        Json(ApiResponse::success(
                            response,
                            "Assignment retrieved successfully",
                        )),
                    )
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<AssignmentResponse>::error(&format!(
                        "Failed to retrieve files: {}",
                        e
                    ))),
                ),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<AssignmentResponse>::error(
                "Assignment not found",
            )),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignmentResponse>::error(&format!(
                "An error occurred: {}",
                e
            ))),
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
// #[derive(Debug, Serialize, Deserialize, FromRow)]
// pub struct AssignmentFile {
//     pub id: i64,
//     pub assignment_id: i64,
//     pub filename: String,
//     pub path: String,
// }

/// GET /api/modules/:module_id/assignments/:assignment_id/files/:file_id
///
/// Download a file from an assignment. Accessible to admins or users assigned to the module.
///
/// ### Response
/// - `200 OK`: Returns file with correct headers
/// - `404 Not Found`: File not found or missing on disk
/// - `500 Internal Server Error`: DB or file access error
pub async fn download_file(
    Path((_module_id, assignment_id, file_id)): Path<(i64, i64, i64)>,
) -> Response {
    let db = db::connect().await;

    let file = match FileEntity::find()
        .filter(FileColumn::Id.eq(file_id as i32))
        .filter(FileColumn::AssignmentId.eq(assignment_id as i32))
        .one(&db)
        .await
    {
        Ok(Some(file)) => file,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("File not found")),
            )
                .into_response();
        }
        Err(err) => {
            eprintln!("DB error fetching file: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error")),
            )
                .into_response();
        }
    };

    let storage_root =
        env::var("ASSIGNMENT_STORAGE_ROOT").unwrap_or_else(|_| "data/assignment_files".to_string());
    let fs_path = PathBuf::from(storage_root).join(&file.path);

    eprintln!("Resolved file path: {:?}", fs_path);

    if tokio::fs::metadata(&fs_path).await.is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("File missing on disk")),
        )
            .into_response();
    }

    let mut file_handle = match FsFile::open(&fs_path).await {
        Ok(f) => f,
        Err(err) => {
            eprintln!("File open error: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Could not open file")),
            )
                .into_response();
        }
    };

    let mut buffer = Vec::new();
    if let Err(err) = file_handle.read_to_end(&mut buffer).await {
        eprintln!("File read error: {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to read file")),
        )
            .into_response();
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", file.filename))
            .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
    );
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );

    (StatusCode::OK, headers, buffer).into_response()
}

pub async fn list_files(Path((module_id, assignment_id)): Path<(i64, i64)>) -> Response {
    let db = db::connect().await;

    // Check if assignment exists
    match AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id as i32))
        .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
        .one(&db)
        .await
    {
        Ok(Some(_)) => {} // assignment exists
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<Vec<File>>::error("Assignment not found")),
            )
                .into_response();
        }
        Err(err) => {
            eprintln!("DB error checking assignment: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<File>>::error("Database error")),
            )
                .into_response();
        }
    }

    // Fetch files for the assignment
    match FileEntity::find()
        .filter(FileColumn::AssignmentId.eq(assignment_id as i32))
        .all(&db)
        .await
    {
        Ok(files) => {
            let file_list: Vec<File> = files
                .into_iter()
                .map(|f| File {
                    id: f.id.to_string(),
                    filename: f.filename,
                    path: f.path,
                    created_at: f.created_at.to_rfc3339(),
                    updated_at: f.updated_at.to_rfc3339(),
                })
                .collect();

            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    file_list,
                    "Assignment files retrieved successfully",
                )),
            )
                .into_response()
        }
        Err(err) => {
            eprintln!("DB error fetching files: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<File>>::error("Failed to retrieve files")),
            )
                .into_response()
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

impl FilterResponse {
    fn new(
        assignments: Vec<AssignmentDetailResponse>,
        page: i32,
        per_page: i32,
        total: i32,
    ) -> Self {
        Self {
            assignments,
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
pub async fn get_assignments(
    Path(module_id): Path<i64>,
    Query(params): Query<FilterReq>,
) -> impl IntoResponse {
    let db: DatabaseConnection = db::connect().await;

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100).max(1);

    // Validate sort fields
    if let Some(sort_field) = &params.sort {
        let valid_fields = [
            "name",
            "description",
            "due_date",
            "available_from",
            "assignment_type",
            "created_at",
            "updated_at",
        ];
        for field in sort_field.split(',') {
            let field = field.trim_start_matches('-');
            if !valid_fields.contains(&field) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<FilterResponse>::error("Invalid field used")),
                );
            }
        }
    }

    // Build filters
    let mut condition = Condition::all().add(AssignmentColumn::ModuleId.eq(module_id as i32));

    if let Some(ref query) = params.query {
        let pattern = format!("%{}%", query.to_lowercase());
        condition = condition.add(
            Condition::any()
                .add(Expr::cust("LOWER(name)").like(&pattern))
                .add(Expr::cust("LOWER(description)").like(&pattern)),
        );
    }

    if let Some(ref name) = params.name {
        let pattern = format!("%{}%", name.to_lowercase());
        condition = condition.add(Expr::cust("LOWER(name)").like(&pattern));
    }

    if let Some(ref assignment_type) = params.assignment_type {
        match assignment_type.parse::<AssignmentType>() {
            Ok(atype_enum) => {
                condition = condition.add(AssignmentColumn::AssignmentType.eq(atype_enum));
            }
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<FilterResponse>::error(
                        "Invalid assignment_type",
                    )),
                );
            }
        }
    }

    if let Some(ref before) = params.available_before {
        if let Ok(date) = DateTime::parse_from_rfc3339(before) {
            condition = condition.add(AssignmentColumn::AvailableFrom.lt(date.with_timezone(&Utc)));
        }
    }

    if let Some(ref after) = params.available_after {
        if let Ok(date) = DateTime::parse_from_rfc3339(after) {
            condition = condition.add(AssignmentColumn::AvailableFrom.gt(date.with_timezone(&Utc)));
        }
    }

    if let Some(ref before) = params.due_before {
        if let Ok(date) = DateTime::parse_from_rfc3339(before) {
            condition = condition.add(AssignmentColumn::DueDate.lt(date.with_timezone(&Utc)));
        }
    }

    if let Some(ref after) = params.due_after {
        if let Ok(date) = DateTime::parse_from_rfc3339(after) {
            condition = condition.add(AssignmentColumn::DueDate.gt(date.with_timezone(&Utc)));
        }
    }

    // Apply base query
    let mut query = AssignmentEntity::find().filter(condition);

    // Apply sorting
    if let Some(sort_param) = &params.sort {
        for sort in sort_param.split(',') {
            let (field, asc) = if sort.starts_with('-') {
                (&sort[1..], false)
            } else {
                (sort, true)
            };

            query = match field {
                "name" => {
                    if asc {
                        query.order_by_asc(AssignmentColumn::Name)
                    } else {
                        query.order_by_desc(AssignmentColumn::Name)
                    }
                }
                "description" => {
                    if asc {
                        query.order_by_asc(AssignmentColumn::Description)
                    } else {
                        query.order_by_desc(AssignmentColumn::Description)
                    }
                }
                "due_date" => {
                    if asc {
                        query.order_by_asc(AssignmentColumn::DueDate)
                    } else {
                        query.order_by_desc(AssignmentColumn::DueDate)
                    }
                }
                "available_from" => {
                    if asc {
                        query.order_by_asc(AssignmentColumn::AvailableFrom)
                    } else {
                        query.order_by_desc(AssignmentColumn::AvailableFrom)
                    }
                }
                "assignment_type" => {
                    if asc {
                        query.order_by_asc(AssignmentColumn::AssignmentType)
                    } else {
                        query.order_by_desc(AssignmentColumn::AssignmentType)
                    }
                }
                "created_at" => {
                    if asc {
                        query.order_by_asc(AssignmentColumn::CreatedAt)
                    } else {
                        query.order_by_desc(AssignmentColumn::CreatedAt)
                    }
                }
                "updated_at" => {
                    if asc {
                        query.order_by_asc(AssignmentColumn::UpdatedAt)
                    } else {
                        query.order_by_desc(AssignmentColumn::UpdatedAt)
                    }
                }
                _ => query,
            };
        }
    }

    // Paginate
    let paginator = query.clone().paginate(&db, per_page as u64);
    let total = match paginator.num_items().await {
        Ok(n) => n as i32,
        Err(e) => {
            eprintln!("Error counting items: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<FilterResponse>::error("Error counting items")),
            );
        }
    };

    match paginator.fetch_page((page - 1) as u64).await {
        Ok(results) => {
            let assignments: Vec<AssignmentDetailResponse> = results
                .into_iter()
                .map(AssignmentDetailResponse::from)
                .collect();

            let response = FilterResponse::new(assignments, page, per_page, total);
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "Assignments retrieved successfully",
                )),
            )
        }
        Err(err) => {
            eprintln!("DB error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<FilterResponse>::error(
                    "Failed to retrieve assignments",
                )),
            )
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SubmissionResponse {
    pub id: i64,
    pub filename: String,
    pub created_at: String,
    pub is_late: bool,
}

/// GET /api/modules/:module_id/assignments/:assignment_id/submissions/me
///
/// Get a list of the current user's submissions for a specific assignment.
///
/// ### Responses
/// - `200 OK` with list of submissions
/// - `404 Not Found` (assignment not found)
/// - `500 Internal Server Error` (database error)
///
pub fn is_late(submission: DateTime<Utc>, due_date: DateTime<Utc>) -> bool {
    submission > due_date
}

pub async fn get_my_submissions(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    let db = connect().await;

    let assignment = match AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id as i32))
        .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
        .one(&db)
        .await
    {
        Ok(Some(a)) => a,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<Vec<SubmissionResponse>>::error(
                    "Assignment not found",
                )),
            )
                .into_response();
        }
        Err(err) => {
            eprintln!("DB error checking assignment: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<SubmissionResponse>>::error(
                    "Database error",
                )),
            )
                .into_response();
        }
    };

    match assignment_submission::Entity::find()
        .filter(assignment_submission::Column::AssignmentId.eq(assignment_id as i32))
        .filter(assignment_submission::Column::UserId.eq(claims.sub as i32))
        .order_by_desc(assignment_submission::Column::CreatedAt)
        .all(&db)
        .await
    {
        Ok(submissions) => {
            let response: Vec<SubmissionResponse> = submissions
                .into_iter()
                .map(|s| SubmissionResponse {
                    id: s.id,
                    filename: s.filename,
                    created_at: s.created_at.to_rfc3339(),
                    is_late: is_late(s.created_at, assignment.due_date),
                })
                .collect();

            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "Submissions retrieved successfully",
                )),
            )
                .into_response()
        }
        Err(err) => {
            eprintln!("DB error fetching submissions: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<SubmissionResponse>>::error(
                    "Failed to retrieve submissions",
                )),
            )
                .into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListSubmissionsQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort: Option<String>,
    pub query: Option<String>,
    pub user_id: Option<i64>,
    pub late: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SubmissionListItem {
    pub id: i64,
    pub user_id: i64,
    pub filename: String,
    pub created_at: String,
    pub is_late: bool,
}

#[derive(Debug, Serialize)]
pub struct SubmissionsListResponse {
    pub submissions: Vec<SubmissionListItem>,
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
}

/// GET /api/modules/:module_id/assignments/:assignment_id/submissions
///
/// Get a paginated list of all submissions for a specific assignment.
/// Only accessible to lecturers or tutors assigned to the module.
///
/// ### Query Parameters
/// - `page` (optional): Page number (default: 1, min: 1)
/// - `per_page` (optional): Items per page (default: 20, min: 1, max: 100)
/// - `query` (optional): Case-insensitive partial match against filename
/// - `user_id` (optional): Filter by user ID
/// - `late` (optional): Filter by late status (true/false)
/// - `sort` (optional): Sort by field. Prefix with `-` for descending. Allowed fields:
///   - `filename`
///   - `created_at`
///   - `user_id`
///
/// ### Responses
/// - `200 OK` with list of submissions
/// - `403 Forbidden` (not a lecturer or tutor)
/// - `404 Not Found` (assignment not found)
/// - `500 Internal Server Error` (database error)
pub async fn list_submissions(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Query(params): Query<ListSubmissionsQuery>,
) -> impl IntoResponse {
    let db = connect().await;

    // Check if assignment exists and get due date
    let assignment = match AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id as i32))
        .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
        .one(&db)
        .await
    {
        Ok(Some(a)) => a,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<SubmissionsListResponse>::error(
                    "Assignment not found",
                )),
            )
                .into_response();
        }
        Err(err) => {
            eprintln!("DB error checking assignment: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<SubmissionsListResponse>::error(
                    "Database error",
                )),
            )
                .into_response();
        }
    };

    // Pagination
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    // Build filter condition
    let mut condition =
        Condition::all().add(assignment_submission::Column::AssignmentId.eq(assignment_id as i32));

    if let Some(user_id) = params.user_id {
        condition = condition.add(assignment_submission::Column::UserId.eq(user_id as i32));
    }

    if let Some(late) = params.late {
        if late {
            condition =
                condition.add(assignment_submission::Column::CreatedAt.gt(assignment.due_date));
        } else {
            condition =
                condition.add(assignment_submission::Column::CreatedAt.lte(assignment.due_date));
        }
    }

    if let Some(ref query) = params.query {
        let pattern = format!("%{}%", query.to_lowercase());
        condition = condition.add(assignment_submission::Column::Filename.contains(&pattern));
    }

    // Build query
    let mut query = assignment_submission::Entity::find().filter(condition);

    // Sorting
    if let Some(ref sort) = params.sort {
        for sort_field in sort.split(',') {
            let (field, dir) = if sort_field.starts_with('-') {
                (&sort_field[1..], sea_orm::Order::Desc)
            } else {
                (sort_field, sea_orm::Order::Asc)
            };

            match field {
                "filename" => query = query.order_by(assignment_submission::Column::Filename, dir),
                "created_at" => {
                    query = query.order_by(assignment_submission::Column::CreatedAt, dir)
                }
                "user_id" => query = query.order_by(assignment_submission::Column::UserId, dir),
                _ => {}
            }
        }
    } else {
        query = query.order_by(
            assignment_submission::Column::CreatedAt,
            sea_orm::Order::Desc,
        );
    }

    // Pagination
    let paginator = query.paginate(&db, per_page.into());
    let total = paginator.num_items().await.unwrap_or(0);
    let submissions = paginator
        .fetch_page((page - 1).into())
        .await
        .unwrap_or_default();

    let response: Vec<SubmissionListItem> = submissions
        .into_iter()
        .map(|s| SubmissionListItem {
            id: s.id,
            user_id: s.user_id,
            filename: s.filename,
            created_at: s.created_at.to_rfc3339(),
            is_late: is_late(s.created_at, assignment.due_date),
        })
        .collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            SubmissionsListResponse {
                submissions: response,
                page,
                per_page,
                total,
            },
            "Submissions retrieved successfully",
        )),
    )
        .into_response()
}

#[derive(Debug, Serialize)]
pub struct PerStudentSubmission {
    pub user_id: i64,
    pub student_number: String,
    pub count: i8,
    pub latest_at: DateTime<Utc>,
    pub latest_late: bool
}


#[derive(Debug, Serialize)]
pub struct StatResponse {
    pub assignment_id: i64,
    pub total_submissions: i8,
    pub unique_submitters: i8,
    pub late_submissions: i8,
    pub per_student_submission_count: Vec<PerStudentSubmission>
}

pub async fn stats(Path((module_id, assignment_id)): Path<(i64, i64)>) -> impl IntoResponse {
    let db = connect().await;

    // Validate assignment exists and get its due date
    let assignment = match AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id as i32))
        .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
        .one(&db)
        .await
    {
        Ok(Some(a)) => a,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<StatResponse>::error("Assignment not found")),
            )
                .into_response();
        }
        Err(err) => {
            eprintln!("DB error fetching assignment: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<StatResponse>::error("Database error")),
            )
                .into_response();
        }
    };

    // Get all submissions for this assignment
    match assignment_submission::Entity::find()
        .filter(assignment_submission::Column::AssignmentId.eq(assignment_id as i32))
        .order_by_desc(assignment_submission::Column::CreatedAt)
        .all(&db)
        .await
    {
        Ok(submissions) => {
            use std::collections::HashMap;

            let mut total_submissions = 0;
            let mut late_submissions = 0;
            let mut unique_users: HashMap<i64, Vec<DateTime<Utc>>> = HashMap::new(); // user_id -> Vec<created_at>

            for sub in &submissions {
                total_submissions += 1;
                if is_late(sub.created_at, assignment.due_date) {
                    late_submissions += 1;
                }

                unique_users
                    .entry(sub.user_id)
                    .or_insert_with(Vec::new)
                    .push(sub.created_at);
            }

            let user_ids: Vec<i64> = unique_users.keys().copied().collect();
            
            let user_models = user::Entity::find()
                .filter(user::Column::Id.is_in(user_ids.clone()))
                .all(&db)
                .await;

            let mut user_id_to_student_number = HashMap::new();
            match user_models {
                Ok(users) => {
                    for user in users {
                        user_id_to_student_number.insert(user.id, user.student_number);
                    }
                }
                Err(err) => {
                    eprintln!("DB error fetching student numbers: {:?}", err);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<StatResponse>::error("Failed to fetch student numbers")),
                    )
                        .into_response();
                }
            }

            let mut per_student_submission_count = Vec::new();

            for (user_id, created_times) in unique_users.iter() {
                let latest_at = *created_times.iter().max().unwrap();
                let latest_late = is_late(latest_at, assignment.due_date);
                let student_number = user_id_to_student_number
                    .get(user_id)
                    .cloned()
                    .unwrap_or_else(|| "UNKNOWN".to_string());

                per_student_submission_count.push(PerStudentSubmission {
                    user_id: *user_id,
                    student_number,
                    count: created_times.len() as i8,
                    latest_at,
                    latest_late,
                });
            }

            let response = StatResponse {
                assignment_id,
                total_submissions,
                unique_submitters: unique_users.len() as i8,
                late_submissions,
                per_student_submission_count,
            };

            (
                StatusCode::OK,
                Json(ApiResponse::success(response, "Stats retrieved successfully")),
            )
                .into_response()
        }
        Err(err) => {
            eprintln!("DB error fetching submissions for stats: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<StatResponse>::error("Database error")),
            )
                .into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    task_number: i64,
    command: String,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    id: i64,
    task_number: i64,
    command: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>
}


pub async fn list_tasks(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let db = connect().await;


    let assignment_exists = AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id as i32))
        .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
        .one(&db)
        .await;

    match assignment_exists {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<Vec<TaskResponse>>::error(
                    "Assignment or module not found",
                )),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<TaskResponse>>::error("Database error")),
            )
                .into_response();
        }
    }


    match Entity::find()
        .filter(Column::AssignmentId.eq(assignment_id))
        .order_by_asc(Column::TaskNumber)
        .all(&db)
        .await
    {
        Ok(tasks) => {
            let data = tasks
                .into_iter()
                .map(|task| TaskResponse {
                    id: task.id,
                    task_number: task.task_number,
                    command: task.command,
                    created_at: task.created_at,
                    updated_at: task.updated_at,
                })
                .collect::<Vec<_>>();

            (
                StatusCode::OK,
                Json(ApiResponse::success(data, "Tasks retrieved successfully")),
            )
                .into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<Vec<TaskResponse>>::error("Failed to retrieve tasks")),
        )
            .into_response(),
    }
}