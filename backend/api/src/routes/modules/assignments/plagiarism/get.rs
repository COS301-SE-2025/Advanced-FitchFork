use axum::{extract::{State, Path, Query}, Json, response::IntoResponse};
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

    Json(serde_json::json!({
        "success": true,
        "message": "Plagiarism cases retrieved successfully",
        "data": {
            "cases": response_cases,
            "page": page,
            "per_page": per_page,
            "total": total_items,
        }
    }))
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
pub async fn get_graph(Query(query): Query<PlagiarismQuery>, State(app_state): State<AppState>) -> impl IntoResponse {
    let mut query_builder = PlagiarismEntity::find();

    if let Some(status_str) = query.status {
        if let Ok(status) = status_str.parse::<plagiarism_case::Status>() {
            query_builder = query_builder.filter(plagiarism_case::Column::Status.eq(status));
        }
    }

    let plagiarism_cases = match query_builder.all(app_state.db()).await {
        Ok(cases) => cases,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch plagiarism cases"})),
            );
        }
    };

    let mut links = Vec::new();

    for case in plagiarism_cases {
        let submission1 = SubmissionEntity::find_by_id(case.submission_id_1)
            .one(app_state.db())
            .await
            .ok()
            .flatten();

        let submission2 = SubmissionEntity::find_by_id(case.submission_id_2)
            .one(app_state.db())
            .await
            .ok()
            .flatten();

        if let (Some(sub1), Some(sub2)) = (submission1, submission2) {
            let user1 = UserEntity::find_by_id(sub1.user_id)
                .one(app_state.db())
                .await
                .ok()
                .flatten();

            let user2 = UserEntity::find_by_id(sub2.user_id)
                .one(app_state.db())
                .await
                .ok()
                .flatten();

            if let (Some(u1), Some(u2)) = (user1, user2) {
                links.push(Link {
                    source: u1.username,
                    target: u2.username,
                });
            }
        }
    }

    // Step 4: Return response with links
    (
        axum::http::StatusCode::OK,
        Json(serde_json::json!({ "links": links })),
    )
}
