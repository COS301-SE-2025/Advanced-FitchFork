use std::{fs, path::PathBuf};
use axum::{extract::{Path, Query}, http::StatusCode, response::IntoResponse, Extension, Json};
use chrono::{DateTime, Utc};
use db::models::{assignment::{Column as AssignmentColumn, Entity as AssignmentEntity}, assignment_submission::{self, Entity as SubmissionEntity}, user, user_module_role::{self, Role}};
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,  QuerySelect, JoinType, RelationTrait};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::{auth::AuthUser, response::ApiResponse};

use db::connect;

fn is_late(submission: DateTime<Utc>, due_date: DateTime<Utc>) -> bool {
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
    pub is_practice: bool,
    pub mark: Option<Mark>,
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
            let base = match std::env::var("ASSIGNMENT_STORAGE_ROOT") {
                Ok(val) => val,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<Vec<SubmissionResponse>>::error(
                            "ASSIGNMENT_STORAGE_ROOT not set",
                        )),
                    )
                    .into_response();
                }
            };

            let response: Vec<SubmissionResponse> = submissions
                .into_iter()
                .map(|s| {
                    let path = PathBuf::from(&base)
                        .join(format!("module_{}", module_id))
                        .join(format!("assignment_{}", assignment_id))
                        .join("assignment_submissions")
                        .join(format!("user_{}", s.user_id))
                        .join(format!("attempt_{}", s.attempt))
                        .join("submission_report.json");

                    let (mark, is_practice) = match fs::read_to_string(&path) {
                        Ok(content) => {
                            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                                let mark = json
                                    .get("mark")
                                    .and_then(|m| serde_json::from_value(m.clone()).ok());
                                let is_practice =
                                    json.get("is_practice").and_then(|p| p.as_bool()).unwrap_or(false);
                                (mark, is_practice)
                            } else {
                                (None, false)
                            }
                        }
                        Err(_) => (None, false),
                    };

                    SubmissionResponse {
                        id: s.id,
                        filename: s.filename,
                        attempt: s.attempt,
                        created_at: s.created_at.to_rfc3339(),
                        updated_at: s.updated_at.to_rfc3339(),
                        is_late: is_late(s.created_at, assignment.due_date),
                        is_practice,
                        mark,
                    }
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
    pub is_practice: bool,
    pub is_late: bool,
    pub mark: Option<Mark>, 
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mark {
    pub earned: i32,
    pub total: i32,
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

    let base = match std::env::var("ASSIGNMENT_STORAGE_ROOT") {
        Ok(val) => val,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<SubmissionListItem>>::error(
                    "ASSIGNMENT_STORAGE_ROOT not set",
                )),
            )
            .into_response();
        }
    };

    let response: Vec<SubmissionListItem> = submissions
        .into_iter()
        .map(|s| {
            let path = PathBuf::from(&base)
                .join(format!("module_{}", module_id))
                .join(format!("assignment_{}", assignment_id))
                .join("assignment_submissions")
                .join(format!("user_{}", s.user_id))
                .join(format!("attempt_{}", s.attempt))
                .join("submission_report.json");

            let (mark, is_practice) = match fs::read_to_string(&path) {
                Ok(content) => {
                    if let Ok(json) = serde_json::from_str::<Value>(&content) {
                        let mark = json
                            .get("mark")
                            .and_then(|m| serde_json::from_value(m.clone()).ok());
                        let is_practice = json
                            .get("is_practice")
                            .and_then(|p| p.as_bool())
                            .unwrap_or(false);
                        (mark, is_practice)
                    } else {
                        (None, false)
                    }
                }
                Err(_) => (None, false),
            };

            SubmissionListItem {
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
                is_practice,
                is_late: is_late(s.created_at, assignment.due_date),
                mark,
            }
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

async fn is_student(module_id: i64, user_id: i64, db: &DatabaseConnection) -> bool {
    user_module_role::Entity::find()
        .filter(user_module_role::Column::UserId.eq(user_id))
        .filter(user_module_role::Column::ModuleId.eq(module_id))
        .filter(user_module_role::Column::Role.eq(Role::Student))
        .join(
        JoinType::InnerJoin,
        user_module_role::Relation::User.def(),
        )
        .filter(user::Column::Admin.eq(false))
        .one(db)
        .await
        .map(|opt| opt.is_some())
        .unwrap_or(false)
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/submissions
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

    if is_student(module_id, user_id, &db).await {
        return get_user_submissions(module_id, assignment_id, user_id)
            .await
            .into_response();
    }

    get_list_submissions(module_id, assignment_id, params)
        .await
        .into_response()
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/submissions/{submission_id}
///
/// Retrieve a specific submission report for a given assignment. Accessible to users assigned to
/// the module with appropriate permissions.
///
/// This endpoint validates that the submission belongs to the specified assignment and module, then
/// reads the `submission_report.json` file associated with it. If the requesting user is not a
/// student, user metadata is included in the response.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment containing the submission
/// - `submission_id` (i64): The ID of the submission to retrieve
///
/// ### Example Request
/// ```bash
/// curl -X GET "http://localhost:3000/api/modules/1/assignments/2/submissions/123" \
///   -H "Authorization: Bearer <token>"
/// ```
///
/// ### Success Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "message": "Submission details retrieved successfully",
///   "data": {
///     "id": 123,
///     "attempt": 1,
///     "filename": "assignment1.java",
///     "created_at": "2024-01-15T10:30:00Z",
///     "updated_at": "2024-01-15T10:30:00Z",
///     "mark": {
///       "earned": 85,
///       "total": 100
///     },
///     "is_practice": false,
///     "is_late": false,
///     "tasks": [...],
///     "code_coverage": [...],
///     "code_complexity": {...},
///     "user": {
///       "user_id": 456,
///       "username": "student1",
///       "email": "student1@example.com"
///     }
///   }
/// }
/// ```
///
/// ### Error Responses
///
/// **404 Not Found** - Submission, assignment, or module not found
/// ```json
/// {
///   "success": false,
///   "message": "Submission not found"
/// }
/// ```
/// or
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
///   "message": "Assignment does not belong to the specified module"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Submission report not found"
/// }
/// ```
///
/// **500 Internal Server Error** - Database or file read error
/// ```json
/// {
///   "success": false,
///   "message": "Database error"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Failed to parse submission report"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "ASSIGNMENT_STORAGE_ROOT not set"
/// }
/// ```
///
/// ### Notes
/// - The submission report is read from the filesystem at:
///   `ASSIGNMENT_STORAGE_ROOT/module_{module_id}/assignment_{assignment_id}/assignment_submissions/user_{user_id}/attempt_{attempt}/submission_report.json`
/// - User metadata is only included for non-student users (lecturers, tutors, admins)
/// - The response contains the complete grading report including marks, tasks, and optional
///   code coverage/complexity analysis
/// - Access is restricted to users with appropriate permissions for the module
pub async fn get_submission(
    Path((module_id, assignment_id, submission_id)): Path<(i64, i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
    let db = connect().await;
    let submission = match SubmissionEntity::find_by_id(submission_id).one(&db).await {
        Ok(Some(sub)) => sub,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Submission not found")),
            )
                .into_response();
        }
        Err(err) => {
            eprintln!("DB error loading submission: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error")),
            )
                .into_response();
        }
    };

    if submission.assignment_id != assignment_id {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(
                "Submission does not belong to the specified assignment",
            )),
        )
            .into_response();
    }

    let assignment = match AssignmentEntity::find_by_id(assignment_id).one(&db).await {
        Ok(Some(assignment)) => assignment,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Assignment not found")),
            )
                .into_response();
        }
        Err(err) => {
            eprintln!("DB error checking assignment: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error")),
            )
                .into_response();
        }
    };
    
    if assignment.module_id != module_id {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(
                "Assignment does not belong to the specified module",
            )),
        )
        .into_response();
    }

    let user_id = submission.user_id;
    let attempt = submission.attempt;

    let base = match std::env::var("ASSIGNMENT_STORAGE_ROOT") {
        Ok(val) => val,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "ASSIGNMENT_STORAGE_ROOT not set",
                )),
            )
            .into_response();
        }
    };

    let path = PathBuf::from(&base)
        .join(format!("module_{}", module_id))
        .join(format!("assignment_{}", assignment_id))
        .join("assignment_submissions")
        .join(format!("user_{}", user_id))
        .join(format!("attempt_{}", attempt))
        .join("submission_report.json");

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error(
                    "Submission report not found",
                )),
            )
            .into_response();
        }
    };

    let mut parsed: Value = match serde_json::from_str(&content) {
        Ok(val) => val,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to parse submission report")),
            )
            .into_response();
        }
    };

    if !is_student(module_id, claims.sub, &db).await {
        if let Ok(Some(u)) = user::Entity::find_by_id(user_id).one(&db).await {
            let user_value = serde_json::to_value(UserResponse {
                user_id: u.id,
                username: u.username,
                email: u.email,
            })
            .unwrap(); // safe since UserResponse is serializable

            if let Some(obj) = parsed.as_object_mut() {
                obj.insert("user".to_string(), user_value);
            }
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            parsed,
            "Submission details retrieved successfully",
        )),
    )
    .into_response()
}