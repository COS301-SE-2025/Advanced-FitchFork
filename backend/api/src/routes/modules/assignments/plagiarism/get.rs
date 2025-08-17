use axum::{extract::{State, Path, Query}, http::StatusCode, Json, response::IntoResponse};
use db::models::{
    assignment_submission::{self, Entity as SubmissionEntity},
    plagiarism_case::{self, Entity as PlagiarismEntity, Status},
    user::{self, Entity as UserEntity},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Condition, QuerySelect, QueryTrait, QueryOrder, PaginatorTrait};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::collections::HashMap;
use util::state::AppState;
use crate::response::ApiResponse;
use std::fs;

#[derive(Serialize)]
pub struct MossReportResponse {
    pub report_url: String,
    pub generated_at: String,
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/plagiarism/moss
pub async fn get_moss_report(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let report_path = assignment_submission::Model::storage_root()
        .join(format!("module_{}", module_id))
        .join(format!("assignment_{}", assignment_id))
        .join("reports.txt");

    if !report_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("MOSS report not found".to_string())),
        )
            .into_response();
    }

    let content = match fs::read_to_string(&report_path) {
        Ok(content) => content,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Failed to read MOSS report: {}", e))),
            )
                .into_response();
        }
    };

    let mut report_url = "".to_string();
    let mut generated_at = "".to_string();

    for line in content.lines() {
        if let Some(url) = line.strip_prefix("Report URL: ") {
            report_url = url.to_string();
        } else if let Some(date) = line.strip_prefix("Date: ") {
            generated_at = date.to_string();
        }
    }

    if report_url.is_empty() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to parse MOSS report".to_string())),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            MossReportResponse {
                report_url,
                generated_at,
            },
            "MOSS report retrieved successfully",
        )),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
pub struct ListPlagiarismCaseQueryParams {
    page: Option<u64>,
    per_page: Option<u64>,
    status: Option<String>,
    query: Option<String>,
    sort: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    id: i64,
    username: String,
    email: String,
    profile_picture_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubmissionResponse {
    id: i64,
    filename: String,
    created_at: chrono::DateTime<chrono::Utc>,
    user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct PlagiarismCaseResponse {
    id: i64,
    status: String,
    description: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    submission_1: SubmissionResponse,
    submission_2: SubmissionResponse,
}

#[derive(Debug, Serialize)]
pub struct PlagiarismCaseListResponse {
    cases: Vec<PlagiarismCaseResponse>,
    page: u64,
    per_page: u64,
    total: u64,
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/plagiarism
///
/// Retrieves paginated plagiarism cases for a specific assignment with filtering and sorting capabilities.
/// Only accessible to lecturers and assistant lecturers assigned to the module.
///
/// # Path Parameters
///
/// - `module_id`: The ID of the parent module
/// - `assignment_id`: The ID of the assignment to retrieve plagiarism cases for
///
/// # Query Parameters
///
/// - `page`: (Optional) Page number for pagination (default: 1, minimum: 1)
/// - `per_page`: (Optional) Number of items per page (default: 20, maximum: 100)
/// - `status`: (Optional) Filter cases by status ("review", "flagged", or "reviewed")
/// - `query`: (Optional) Case-insensitive fuzzy search on usernames of either submission's user
/// - `sort`: (Optional) Comma-separated sorting criteria. Prefix with '-' for descending order.
///   Valid fields: "created_at", "status"
///
/// # Returns
///
/// Returns an HTTP response indicating the result:
/// - `200 OK` with paginated plagiarism cases if successful
/// - `400 BAD REQUEST` for invalid parameters (status, sort, pagination values)
/// - `403 FORBIDDEN` if user lacks required permissions
/// - `500 INTERNAL SERVER ERROR` for database errors or data retrieval failures
///
/// The response body follows a standardized JSON format containing:
/// - Pagination metadata (current page, items per page, total items)
/// - List of plagiarism cases with enriched submission and user details
///
/// # Example Response (200 OK)
///
/// ```json
/// {
///   "success": true,
///   "message": "Plagiarism cases retrieved successfully",
///   "data": {
///     "cases": [
///       {
///         "id": 12,
///         "status": "flagged",
///         "description": "Very similar submissions",
///         "created_at": "2024-05-15T08:30:00Z",
///         "updated_at": "2024-05-16T10:15:00Z",
///         "submission_1": {
///           "id": 42,
///           "filename": "main.cpp",
///           "created_at": "2024-05-14T09:00:00Z",
///           "user": {
///             "id": 5,
///             "username": "u12345678",
///             "email": "abc@example.com",
///             "profile_picture_path": null
///           }
///         },
///         "submission_2": {
///           "id": 43,
///           "filename": "main.cpp",
///           "created_at": "2024-05-14T10:30:00Z",
///           "user": {
///             "id": 6,
///             "username": "u98765432",
///             "email": "xyz@example.com",
///             "profile_picture_path": null
///           }
///         }
///       }
///     ],
///     "page": 1,
///     "per_page": 20,
///     "total": 1
///   }
/// }
/// ```
///
/// # Example Responses
///
/// - `400 Bad Request` (invalid status)  
/// ```json
/// {
///   "success": false,
///   "message": "Invalid status parameter"
/// }
/// ```
///
/// - `403 Forbidden` (unauthorized role)  
/// ```json
/// {
///   "success": false,
///   "message": "Forbidden: Insufficient permissions"
/// }
/// ```
///
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "message": "Failed to retrieve plagiarism cases"
/// }
/// ```
pub async fn list_plagiarism_cases(
    State(app_state): State<AppState>,
    Path((_, assignment_id)): Path<(i64, i64)>,
    Query(params): Query<ListPlagiarismCaseQueryParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20).min(100);

    let submission_models = SubmissionEntity::find()
        .filter(assignment_submission::Column::AssignmentId.eq(assignment_id))
        .all(app_state.db())
        .await
        .unwrap_or_default();
    
    let submission_ids: Vec<i64> = submission_models.iter().map(|s| s.id).collect();

    let mut query = PlagiarismEntity::find()
        .filter(
            Condition::any()
                .add(plagiarism_case::Column::SubmissionId1.is_in(submission_ids.clone()))
                .add(plagiarism_case::Column::SubmissionId2.is_in(submission_ids))
        );

    if let Some(status_str) = params.status {
        if let Ok(status) = Status::from_str(&status_str) {
            query = query.filter(plagiarism_case::Column::Status.eq(status));
        } else {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<PlagiarismCaseListResponse>::error("Invalid status parameter")),
            );
        }
    }
    if let Some(search_query) = params.query {
        let user_ids_subquery = UserEntity::find()
            .select_only()
            .column(user::Column::Id)
            .filter(user::Column::Username.like(format!("%{}%", search_query.to_lowercase())))
            .into_query();

        let submission_ids_subquery = SubmissionEntity::find()
            .select_only()
            .column(assignment_submission::Column::Id)
            .filter(assignment_submission::Column::UserId.in_subquery(user_ids_subquery))
            .into_query();
        
        query = query.filter(
            Condition::any()
                .add(plagiarism_case::Column::SubmissionId1.in_subquery(submission_ids_subquery.clone()))
                .add(plagiarism_case::Column::SubmissionId2.in_subquery(submission_ids_subquery))
        );
    }

    if let Some(sort) = params.sort {
        for s in sort.split(',') {
            let (order, column) = if s.starts_with('-') {
                (sea_orm::Order::Desc, &s[1..])
            } else {
                (sea_orm::Order::Asc, s)
            };

            match column {
                "created_at" => query = query.order_by(plagiarism_case::Column::CreatedAt, order),
                "status" => query = query.order_by(plagiarism_case::Column::Status, order),
                _ => {}
            }
        }
    }

    let paginator = query.paginate(app_state.db(), per_page);
    let total_items = paginator.num_items().await.unwrap_or(0);
    let cases = paginator.fetch_page(page - 1).await.unwrap_or_default();

    let submission_ids: Vec<i64> = cases
        .iter()
        .flat_map(|case| vec![case.submission_id_1, case.submission_id_2])
        .collect();

    let submissions = SubmissionEntity::find()
        .filter(assignment_submission::Column::Id.is_in(submission_ids))
        .all(app_state.db())
        .await
        .unwrap_or_default();

    let user_ids: Vec<i64> = submissions.iter().map(|s| s.user_id).collect();
    let users = UserEntity::find()
        .filter(user::Column::Id.is_in(user_ids))
        .all(app_state.db())
        .await
        .unwrap_or_default();

    let user_map: HashMap<i64, user::Model> =
        users.into_iter().map(|u| (u.id, u)).collect();

    let submission_map: HashMap<i64, (assignment_submission::Model, user::Model)> = submissions
        .into_iter()
        .filter_map(|s| {
            user_map
                .get(&s.user_id)
                .cloned()
                .map(|u| (s.id, (s, u)))
        })
        .collect();

    let response_cases: Vec<PlagiarismCaseResponse> = cases
        .into_iter()
        .filter_map(|case| {
            let sub1_data = submission_map.get(&case.submission_id_1)?;
            let sub2_data = submission_map.get(&case.submission_id_2)?;

            Some(PlagiarismCaseResponse {
                id: case.id,
                status: case.status.to_string(),
                description: case.description,
                created_at: case.created_at,
                updated_at: case.updated_at,
                submission_1: SubmissionResponse {
                    id: sub1_data.0.id,
                    filename: sub1_data.0.filename.clone(),
                    created_at: sub1_data.0.created_at,
                    user: UserResponse {
                        id: sub1_data.1.id,
                        username: sub1_data.1.username.clone(),
                        email: sub1_data.1.email.clone(),
                        profile_picture_path: sub1_data.1.profile_picture_path.clone(),
                    },
                },
                submission_2: SubmissionResponse {
                    id: sub2_data.0.id,
                    filename: sub2_data.0.filename.clone(),
                    created_at: sub2_data.0.created_at,
                    user: UserResponse {
                        id: sub2_data.1.id,
                        username: sub2_data.1.username.clone(),
                        email: sub2_data.1.email.clone(),
                        profile_picture_path: sub2_data.1.profile_picture_path.clone(),
                    },
                },
            })
        })
        .collect();

    let response = PlagiarismCaseListResponse {
        cases: response_cases,
        page,
        per_page,
        total: total_items,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(response, "Plagiarism cases retrieved successfully")),
    )
}

#[derive(Debug, Deserialize)]
pub struct PlagiarismQuery {
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Link {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Serialize)]
pub struct LinksResponse {
    pub links: Vec<Link>,
}

// TODO: Testing and docs @Aidan
pub async fn get_graph(
    State(app_state): State<AppState>,
    Path((_module_id, assignment_id)): Path<(i64, i64)>,
    Query(query): Query<PlagiarismQuery>,
) -> impl IntoResponse {
    // 1) Gather all submission IDs for this assignment
    let submission_models = match SubmissionEntity::find()
        .filter(assignment_submission::Column::AssignmentId.eq(assignment_id))
        .all(app_state.db())
        .await
    {
        Ok(list) => list,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<LinksResponse>::error("Failed to fetch submissions")),
            );
        }
    };

    let assignment_submission_ids: Vec<i64> = submission_models.iter().map(|s| s.id).collect();

    // 2) Base query: plagiarism cases where either side belongs to this assignment
    let mut q = PlagiarismEntity::find().filter(
        Condition::any()
            .add(plagiarism_case::Column::SubmissionId1.is_in(assignment_submission_ids.clone()))
            .add(plagiarism_case::Column::SubmissionId2.is_in(assignment_submission_ids.clone())),
    );

    // 3) Optional status filter
    if let Some(status_str) = query.status {
        match Status::try_from(status_str.as_str()) {
            Ok(status) => {
                q = q.filter(plagiarism_case::Column::Status.eq(status));
            }
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<LinksResponse>::error("Invalid status parameter")),
                );
            }
        }
    }

    // 4) Fetch cases
    let cases = match q.all(app_state.db()).await {
        Ok(cs) => cs,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<LinksResponse>::error("Failed to fetch plagiarism cases")),
            );
        }
    };

    if cases.is_empty() {
        return (
            StatusCode::OK,
            Json(ApiResponse::success(
                LinksResponse { links: vec![] },
                "Plagiarism graph retrieved successfully",
            )),
        );
    }

    // 5) Fetch the submissions & users referenced by these cases
    let all_sub_ids: Vec<i64> = cases
        .iter()
        .flat_map(|c| [c.submission_id_1, c.submission_id_2])
        .collect();

    let submissions = match SubmissionEntity::find()
        .filter(assignment_submission::Column::Id.is_in(all_sub_ids))
        .all(app_state.db())
        .await
    {
        Ok(ss) => ss,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<LinksResponse>::error("Failed to fetch submissions for cases")),
            );
        }
    };

    let user_ids: Vec<i64> = submissions.iter().map(|s| s.user_id).collect();
    let users = match UserEntity::find()
        .filter(user::Column::Id.is_in(user_ids))
        .all(app_state.db())
        .await
    {
        Ok(us) => us,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<LinksResponse>::error("Failed to fetch users")),
            );
        }
    };

    let sub_by_id: HashMap<i64, _> = submissions.into_iter().map(|s| (s.id, s)).collect();
    let user_by_id: HashMap<i64, _> = users.into_iter().map(|u| (u.id, u)).collect();

    // 6) Build username links
    let mut links = Vec::with_capacity(cases.len());
    for case in cases {
        if let (Some(sub1), Some(sub2)) = (
            sub_by_id.get(&case.submission_id_1),
            sub_by_id.get(&case.submission_id_2),
        ) {
            if let (Some(u1), Some(u2)) = (user_by_id.get(&sub1.user_id), user_by_id.get(&sub2.user_id)) {
                links.push(Link {
                    source: u1.username.clone(),
                    target: u2.username.clone(),
                });
            }
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            LinksResponse { links },
            "Plagiarism graph retrieved successfully",
        )),
    )
}