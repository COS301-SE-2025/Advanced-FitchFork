use std::fs;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use db::{
    connect,
    models::{
        assignment::{Column as AssignmentColumn, Entity as AssignmentEntity},
        assignment_submission::{self, Entity as SubmissionEntity},
        user,
        user_module_role::{self, Role},
    },
};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,  QuerySelect, JoinType, RelationTrait
};
use serde_json::Value;
use crate::{auth::AuthUser, response::ApiResponse};
use super::common::{
    ListSubmissionsQuery, UserResponse, SubmissionListItem,
    SubmissionsListResponse,
};

fn is_late(submission: DateTime<Utc>, due_date: DateTime<Utc>) -> bool {
    submission > due_date
}

/// GET /api/modules/:module_id/assignments/:assignment_id/submissions
///
/// Retrieve the list of submissions for a specific assignment **belonging to the authenticated student only**.
/// This endpoint enforces access control such that the user only sees their own submissions, regardless of query params.
///
/// ### When to use
/// - Called when a student views their own submissions for an assignment.
/// - Similar to `get_list_submissions` but limited to `user_id`.
///
/// ### Path Parameters
/// - `module_id` (i64): ID of the module.
/// - `assignment_id` (i64): ID of the assignment.
/// - `user_id` (i64): ID of the student (extracted from JWT claims).
///
/// ### Query Parameters
/// - `page` (optional): Page number (default 1, min 1).
/// - `per_page` (optional): Items per page (default 20, max 100).
/// - `query` (optional): Case-insensitive partial match on filename.
/// - `sort` (optional): Comma-separated sort fields (`created_at`, `filename`, `attempt`, `mark`).
///
/// ### Sorting
/// - DB-backed sorting is available on `created_at`, `filename`, `attempt`.
/// - In-memory sorting is applied for `mark` after file is read.
///
/// ### Output
/// - Returns a paginated `SubmissionsListResponse` for this student, including `is_late`, `mark`, `is_practice`.
/// - Late status is computed relative to assignment `due_date`.
///
/// ### Errors
/// - `404`: Assignment not found.
/// - `500`: DB error.
///
/// ### Notes
/// - No filtering on other students or usernames is possible in this endpoint.
async fn get_user_submissions(
    module_id: i64,
    assignment_id: i64,
    user_id: i64,
    Query(params): Query<ListSubmissionsQuery>,
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
                Json(ApiResponse::<SubmissionsListResponse>::error(
                    "Assignment not found",
                )),
            )
                .into_response();
        }
        Err(_) => {
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
        Condition::all().add(assignment_submission::Column::AssignmentId.eq(assignment_id as i32))
                        .add(assignment_submission::Column::UserId.eq(user_id));

    if let Some(query) = &params.query {
        let pattern = format!("%{}%", query.to_lowercase());
        condition = condition.add(assignment_submission::Column::Filename.contains(&pattern));
    }

    let mut query = assignment_submission::Entity::find()
        .filter(condition);

    if let Some(ref sort) = params.sort {
        for field in sort.split(',') {
            let (field, dir) = if field.starts_with('-') {
                (&field[1..], sea_orm::Order::Desc)
            } else {
                (field, sea_orm::Order::Asc)
            };

            match field {
                "created_at" => query = query.order_by(assignment_submission::Column::CreatedAt, dir),
                "filename" => query = query.order_by(assignment_submission::Column::Filename, dir),
                "attempt" => query = query.order_by(assignment_submission::Column::Attempt, dir),
                _ => {}
            }
        }
    } else {
        query = query.order_by(assignment_submission::Column::CreatedAt, sea_orm::Order::Desc);
    }

    let paginator = query.paginate(&db, per_page.into());
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page((page - 1) as u64)
        .await
        .unwrap_or_default();

    let base = std::env::var("ASSIGNMENT_STORAGE_ROOT").unwrap_or_else(|_| "data/assignment_files".into());

    let user_resp = {
        let u = user::Entity::find_by_id(user_id).one(&db).await.ok().flatten();
        if let Some(u) = u {
            UserResponse {
                id: u.id,
                username: u.username,
                email: u.email,
            }
        } else {
            UserResponse {
                id: user_id,
                username: "unknown".to_string(),
                email: "unknown".to_string(),
            }
        }
    };

    let mut items: Vec<SubmissionListItem> = rows
        .into_iter()
        .map(|s| {
            let report_path = PathBuf::from(&base)
                .join(format!("module_{module_id}"))
                .join(format!("assignment_{assignment_id}"))
                .join("assignment_submissions")
                .join(format!("user_{}", s.user_id))
                .join(format!("attempt_{}", s.attempt))
                .join("submission_report.json");

            let (mark, is_practice) = match fs::read_to_string(&report_path) {
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
                user: user_resp.clone(),
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

    if let Some(ref sort) = params.sort {
        for field in sort.split(',').rev() {
            let (field, desc) = if field.starts_with('-') {
                (&field[1..], true)
            } else {
                (field, false)
            };

            match field {
                "mark" => {
                    items.sort_by(|a, b| {
                        let a_mark = a.mark.as_ref().map(|m| m.earned).unwrap_or(0);
                        let b_mark = b.mark.as_ref().map(|m| m.earned).unwrap_or(0);
                        let ord = a_mark.cmp(&b_mark);
                        if desc { ord.reverse() } else { ord }
                    });
                }
                _ => {}
            }
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            SubmissionsListResponse {
                submissions: items,
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
/// Retrieve the full list of submissions for a specific assignment.  
/// Accessible to lecturers, tutors, and admins who have permission on the module.  
/// Supports advanced filtering, sorting, and pagination over all students.
///
/// ### When to use
/// - Called when a lecturer or tutor wants to review or grade submissions from all students.
///
/// ### Path Parameters
/// - `module_id` (i64): ID of the module.
/// - `assignment_id` (i64): ID of the assignment.
///
/// ### Query Parameters
/// - `page` (optional): Page number (default 1, min 1).
/// - `per_page` (optional): Items per page (default 20, max 100).
/// - `query` (optional): Case-insensitive partial match on filename or username.
/// - `username` (optional): Filter submissions of a specific student.
/// - `sort` (optional): Comma-separated sort fields. Prefix with `-` for descending. Allowed fields:
///   - `username`
///   - `attempt`
///   - `filename`
///   - `created_at`
///   - `mark`
///
/// ### Sorting
/// - DB-backed sorting for `created_at`, `filename`, `attempt`.
/// - In-memory sorting for `username` and `mark`.
///
/// ### Filtering
/// - If `query` is provided, applies to both `filename` and `username`.
/// - If `username` is provided, restricts to that user.
///
/// ### Output
/// - Returns paginated `SubmissionsListResponse` for all students, including `is_late`, `mark`, `is_practice`.
///
/// ### Errors
/// - `404`: Assignment not found.
/// - `500`: DB error.
///
/// ### Notes
/// - Late submissions are computed relative to assignment `due_date`.
/// - Defaults to sorting by `created_at DESC`.
async fn get_list_submissions(
    module_id: i64,
    assignment_id: i64,
    params: ListSubmissionsQuery,
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
                Json(ApiResponse::<SubmissionsListResponse>::error(
                    "Assignment not found",
                )),
            )
                .into_response();
        }
        Err(_) => {
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

    if let Some(query) = &params.query {
        let pattern = format!("%{}%", query.to_lowercase());

        // Build OR condition across multiple fields
        let mut or_condition = Condition::any()
            .add(assignment_submission::Column::Filename.contains(&pattern));

        if let Ok(Some(user)) = user::Entity::find()
            .filter(user::Column::Username.contains(&pattern))
            .one(&db)
            .await
        {
            or_condition = or_condition.add(assignment_submission::Column::UserId.eq(user.id));
        }

        condition = condition.add(or_condition);
    }

    if let Some(ref username) = params.username {
        match user::Entity::find()
            .filter(user::Column::Username.eq(username.clone()))
            .one(&db)
            .await
        {
            Ok(Some(user)) => {
                condition = condition.add(assignment_submission::Column::UserId.eq(user.id));
            }
            Ok(None) => {
                // No such user => return empty list
                return (
                    StatusCode::OK,
                    Json(ApiResponse::success(
                        SubmissionsListResponse {
                            submissions: vec![],
                            page: 1,
                            per_page,
                            total: 0,
                        },
                        "No submissions found for the specified username",
                    )),
                )
                    .into_response();
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<SubmissionsListResponse>::error("Database error")),
                )
                    .into_response();
            }
        }
    }

    let mut query = assignment_submission::Entity::find()
        .filter(condition)
        .find_also_related(user::Entity);

    if let Some(ref sort) = params.sort {
        for field in sort.split(',') {
            let (field, dir) = if field.starts_with('-') {
                (&field[1..], sea_orm::Order::Desc)
            } else {
                (field, sea_orm::Order::Asc)
            };

            match field {
                "created_at" => query = query.order_by(assignment_submission::Column::CreatedAt, dir),
                "filename" => query = query.order_by(assignment_submission::Column::Filename, dir),
                "attempt" => query = query.order_by(assignment_submission::Column::Attempt, dir),
                _ => {} // mark/status handled in-memory
            }
        }
    } else {
        query = query.order_by(assignment_submission::Column::CreatedAt, sea_orm::Order::Desc);
    }

    let paginator = query.paginate(&db, per_page.into());
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator
        .fetch_page((page - 1) as u64)
        .await
        .unwrap_or_default();

    let base = std::env::var("ASSIGNMENT_STORAGE_ROOT").unwrap_or_else(|_| "data/assignment_files".into());

    let mut items: Vec<SubmissionListItem> = rows
        .into_iter()
        .map(|(s, u)| {
            let user_resp = if let Some(u) = u {
                UserResponse {
                    id: u.id,
                    username: u.username,
                    email: u.email,
                }
            } else {
                UserResponse {
                    id: s.user_id,
                    username: "unknown".to_string(),
                    email: "unknown".to_string(),
                }
            };

            let report_path = PathBuf::from(&base)
                .join(format!("module_{module_id}"))
                .join(format!("assignment_{assignment_id}"))
                .join("assignment_submissions")
                .join(format!("user_{}", s.user_id))
                .join(format!("attempt_{}", s.attempt))
                .join("submission_report.json");

            let (mark, is_practice) = match fs::read_to_string(&report_path) {
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
                user: user_resp,
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

    if let Some(ref sort) = params.sort {
        for field in sort.split(',').rev() {
            let (field, desc) = if field.starts_with('-') {
                (&field[1..], true)
            } else {
                (field, false)
            };

            match field {
                "username" => {
                    items.sort_by(|a, b| {
                        let ord = a.user.username.cmp(&b.user.username);
                        if desc { ord.reverse() } else { ord }
                    });
                }
                "mark" => {
                    items.sort_by(|a, b| {
                        let a_mark = a.mark.as_ref().map(|m| m.earned).unwrap_or(0);
                        let b_mark = b.mark.as_ref().map(|m| m.earned).unwrap_or(0);
                        let ord = a_mark.cmp(&b_mark);
                        if desc { ord.reverse() } else { ord }
                    });
                }
                _ => {}
            }
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            SubmissionsListResponse {
                submissions: items,
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
/// List submissions for a specific assignment.  
/// - **Students**: Can only view their own submissions, with optional query, pagination, and sort.  
/// - **Lecturers/Tutors/Admins**: Can view all submissions with full filters, pagination, sorting.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module
/// - `assignment_id` (i64): The ID of the assignment
///
/// ### Query Parameters
/// - `page`, `per_page`, `query`, `username`, `sort` (see API docs above for details)
///
/// ### Notes
/// - Students: `username` is ignored, only their own submissions returned.
/// - Late submissions are calculated based on `due_date`.
pub async fn list_submissions(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Query(params): Query<ListSubmissionsQuery>,
) -> axum::response::Response {
    let user_id = claims.sub;
    let db = connect().await;

    if is_student(module_id, user_id, &db).await {
        // Pass params explicitly now
        return get_user_submissions(module_id, assignment_id, user_id, Query(params))
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
                id: u.id,
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