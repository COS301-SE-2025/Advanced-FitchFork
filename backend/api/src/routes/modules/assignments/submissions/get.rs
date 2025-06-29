use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use chrono::{DateTime, Utc};
use db::{
    connect,
    models::{
        assignment::{Column as AssignmentColumn, Entity as AssignmentEntity},
        assignment_submission, user,
        user_module_role::{self, Role},
    },
};
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};

use crate::{auth::AuthUser, response::ApiResponse};

pub fn is_late(submission: DateTime<Utc>, due_date: DateTime<Utc>) -> bool {
    submission > due_date
}

#[derive(Debug, Serialize)]
pub struct SubmissionResponse {
    pub id: i64,
    pub attempt: i64,
    pub filename: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_late: bool,
}

async fn get_user_submissions(
    module_id: i64,
    assignment_id: i64,
    user_id: i64,
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
        .filter(assignment_submission::Column::UserId.eq(user_id))
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
                    attempt: s.attempt,
                    created_at: s.created_at.to_rfc3339(),
                    updated_at: s.updated_at.to_rfc3339(),
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
    pub username: Option<String>,
}
#[derive(Debug, Serialize, Clone)]
pub struct UserResponse {
    user_id: i64,
    username: String,
    email: String,
}

#[derive(Debug, Serialize)]
pub struct SubmissionListItem {
    pub id: i64,
    pub user: UserResponse,
    pub attempt: i64,
    pub filename: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_late: bool,
}

#[derive(Debug, Serialize)]
pub struct SubmissionsListResponse {
    pub submissions: Vec<SubmissionListItem>,
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
}

async fn get_list_submissions(
    module_id: i64,
    assignment_id: i64,
    params: ListSubmissionsQuery,
) -> impl IntoResponse {
    let db = connect().await;
    if let Some(username) = &params.username {
        match user::Entity::find()
            .filter(user::Column::Username.eq(username.clone()))
            .one(&db)
            .await
        {
            Ok(Some(user)) => {
                return get_user_submissions(module_id, assignment_id, user.id)
                    .await
                    .into_response();
            }
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::<SubmissionsListResponse>::error(
                        "Student not found",
                    )),
                )
                    .into_response();
            }
            Err(err) => {
                eprintln!("DB error looking up student_number: {:?}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<SubmissionsListResponse>::error(
                        "Database error",
                    )),
                )
                    .into_response();
            }
        }
    }

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

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

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

    let mut query = assignment_submission::Entity::find().filter(condition);

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

    let paginator = query.paginate(&db, per_page.into());
    let total = paginator.num_items().await.unwrap_or(0);
    let submissions = paginator
        .fetch_page((page - 1).into())
        .await
        .unwrap_or_default();

    let user_ids: Vec<i64> = submissions.iter().map(|s| s.user_id).collect();

    let users = user::Entity::find()
        .filter(user::Column::Id.is_in(user_ids.clone()))
        .all(&db)
        .await
        .unwrap_or_default();

    let user_map: std::collections::HashMap<i64, UserResponse> = users
        .into_iter()
        .map(|u| {
            (
                u.id,
                UserResponse {
                    user_id: u.id,
                    username: u.username,
                    email: u.email,
                },
            )
        })
        .collect();

    let response: Vec<SubmissionListItem> = submissions
        .into_iter()
        .map(|s| SubmissionListItem {
            id: s.id,
            user: user_map.get(&s.user_id).cloned().unwrap_or(UserResponse {
                user_id: s.user_id,
                username: "unknown username".to_string(),
                email: "unknown email".to_string(),
            }),
            filename: s.filename,
            attempt: s.attempt,
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
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

/// GET /api/modules/:module_id/assignments/:assignment_id/submissions
///
/// List submissions for a specific assignment. Accessible to users assigned to the module with
/// appropriate permissions.
///
/// This endpoint provides different behavior based on the user's role:
/// - **Students**: Can only view their own submissions for the assignment
/// - **Lecturers/Tutors**: Can view all submissions with filtering and pagination options
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to list submissions for
///
/// ### Query Parameters
/// - `page` (optional, u32): Page number (default: 1, min: 1)
/// - `per_page` (optional, u32): Items per page (default: 20, min: 1, max: 100)
/// - `query` (optional, string): Case-insensitive partial match against filename
/// - `user_id` (optional, i64): Filter by specific user ID
/// - `late` (optional, boolean): Filter by late status (true/false)
/// - `username` (optional, string): Filter by username (returns only that user's submissions)
/// - `sort` (optional, string): Sort by field. Prefix with `-` for descending. Allowed fields:
///   - `filename` - Sort by submission filename
///   - `created_at` - Sort by submission timestamp
///   - `user_id` - Sort by user ID
///
/// ### Example Request - Student View
/// ```bash
/// curl -X GET "http://localhost:3000/api/modules/1/assignments/2/submissions" \
///   -H "Authorization: Bearer <token>"
/// ```
///
/// ### Example Request - Lecturer View with Filters
/// ```bash
/// curl -X GET "http://localhost:3000/api/modules/1/assignments/2/submissions?page=1&per_page=10&late=true&sort=-created_at" \
///   -H "Authorization: Bearer <token>"
/// ```
///
/// ### Success Response (200 OK) - Student View
/// ```json
/// {
///   "success": true,
///   "message": "Submissions retrieved successfully",
///   "data": [
///     {
///       "id": 123,
///       "attempt": 1,
///       "filename": "assignment1.java",
///       "created_at": "2024-01-15T10:30:00Z",
///       "updated_at": "2024-01-15T10:30:00Z",
///       "is_late": false
///     },
///     {
///       "id": 124,
///       "attempt": 2,
///       "filename": "assignment1_v2.java",
///       "created_at": "2024-01-16T14:45:00Z",
///       "updated_at": "2024-01-16T14:45:00Z",
///       "is_late": true
///     }
///   ]
/// }
/// ```
///
/// ### Success Response (200 OK) - Lecturer View
/// ```json
/// {
///   "success": true,
///   "message": "Submissions retrieved successfully",
///   "data": {
///     "submissions": [
///       {
///         "id": 123,
///         "user": {
///           "user_id": 456,
///           "username": "student1",
///           "email": "student1@example.com"
///         },
///         "attempt": 1,
///         "filename": "assignment1.java",
///         "created_at": "2024-01-15T10:30:00Z",
///         "updated_at": "2024-01-15T10:30:00Z",
///         "is_late": false
///       }
///     ],
///     "page": 1,
///     "per_page": 10,
///     "total": 25
///   }
/// }
/// ```
///
/// ### Error Responses
///
/// **403 Forbidden** - Insufficient permissions
/// ```json
/// {
///   "success": false,
///   "message": "Access denied"
/// }
/// ```
///
/// **404 Not Found** - Assignment or student not found
/// ```json
/// {
///   "success": false,
///   "message": "Assignment not found"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Student not found"
/// }
/// ```
///
/// **500 Internal Server Error** - Database error
/// ```json
/// {
///   "success": false,
///   "message": "Database error"
/// }
/// ```
///
/// ### Role-Based Access
/// - **Students**: Can only view their own submissions for the assignment
/// - **Lecturers/Tutors**: Can view all submissions with full filtering and pagination
/// - **Admins**: Have the same access as lecturers/tutors
///
/// ### Late Submission Detection
/// Submissions are marked as late if they are submitted after the assignment's due date.
/// The `is_late` field is calculated by comparing the submission timestamp with the assignment's
/// `due_date` field.
///
/// ### Notes
/// - Students automatically see only their own submissions regardless of query parameters
/// - Lecturers and tutors can use all filtering and pagination options
/// - The `username` parameter overrides other filters and returns only that user's submissions
/// - Sorting defaults to `created_at` in descending order (newest first)
/// - Pagination is applied only for lecturer/tutor views
/// - File information includes attempt numbers for tracking multiple submissions
pub async fn list_submissions(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Query(params): Query<ListSubmissionsQuery>,
) -> axum::response::Response {
    let user_id = claims.sub;
    let db = connect().await;
    let is_student = user_module_role::Entity::find()
        .filter(user_module_role::Column::UserId.eq(user_id))
        .filter(user_module_role::Column::ModuleId.eq(module_id))
        .filter(user_module_role::Column::Role.eq(Role::Student))
        .one(&db)
        .await
        .map(|opt| opt.is_some())
        .unwrap_or(false);
    if is_student {
        return get_user_submissions(module_id, assignment_id, user_id)
            .await
            .into_response();
    }

    get_list_submissions(module_id, assignment_id, params)
        .await
        .into_response()
}