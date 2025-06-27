use std::fs;

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
        assignment_submission::{self, Entity as SubmissionEntity},
        user,
        user_module_role::{self, Role},
    },
};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,  QuerySelect, JoinType, RelationTrait
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{auth::AuthUser, response::ApiResponse};

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

pub async fn get_user_submissions(
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
                .map(|s| {
                    let (mark, is_practice) = {
                        let path = format!(
                            "./data/assignment_files/module_{}/assignment_{}/assignment_submissions/user_{}/attempt_{}/submission_report.json",
                            module_id, assignment_id, s.user_id, s.attempt
                        );

                        match fs::read_to_string(&path) {
                            Ok(content) => {
                                if let Ok(json) = serde_json::from_str::<Value>(&content) {
                                    let mark = json.get("mark").and_then(|m| serde_json::from_value(m.clone()).ok());
                                    let is_practice = json.get("is_practice").and_then(|p| p.as_bool()).unwrap_or(false);
                                    (mark, is_practice)
                                } else {
                                    (None, false)
                                }
                            }
                            Err(_) => (None, false),
                        }
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
///   - `username` filter by unique students
/// ### Responses
/// - `200 OK` with list of submissions
/// - `403 Forbidden` (not a lecturer or tutor)
/// - `404 Not Found` (assignment not found)
/// - `500 Internal Server Error` (database error)
///
pub async fn get_list_submissions(
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
        .map(|s| {
            let (mark, is_practice) = {
                let path = format!(
                    "./data/assignment_files/module_{}/assignment_{}/assignment_submissions/user_{}/attempt_{}/submission_report.json",
                    module_id, assignment_id, s.user_id, s.attempt
                );

                match fs::read_to_string(&path) {
                    Ok(content) => {
                        if let Ok(json) = serde_json::from_str::<Value>(&content) {
                            let mark = json.get("mark").and_then(|m| serde_json::from_value(m.clone()).ok());
                            let is_practice = json.get("is_practice").and_then(|p| p.as_bool()).unwrap_or(false);
                            (mark, is_practice)
                        } else {
                            (None, false)
                        }
                    }
                    Err(_) => (None, false),
                }
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

pub async fn is_student(module_id: i64, user_id: i64, db: &DatabaseConnection) -> bool {
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

/// Get a specific submission report for a given assignment.
///
/// Validates that the submission belongs to the specified assignment and module, then reads the
/// `submission_report.json` file associated with it. If the requesting user is not a student,
/// user metadata is included in the response.
///
/// ### Path Parameters
/// - `module_id`: ID of the module
/// - `assignment_id`: ID of the assignment
/// - `submission_id`: ID of the submission
///
/// ### Behavior
/// - Validates assignment and module linkage
/// - Reads and parses `submission_report.json` from disk
/// - If requester is not a student, adds user info (`user_id`, `student_number`, `email`) to the response
///
/// ### Responses
/// - `200 OK` with the submission report
/// - `404 Not Found` if submission, assignment, or module mismatch
/// - `500 Internal Server Error` if database or file read fails

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
    let path = format!("./data/assignment_files/module_{}/assignment_{}/assignment_submissions/user_{}/attempt_{}/submission_report.json", module_id, assignment_id, user_id, attempt);
    let content = fs::read_to_string(&path).unwrap();
    let mut parsed: Value = serde_json::from_str(&content).unwrap();

    if !is_student(module_id, claims.sub, &db).await {
        let user = user::Entity::find_by_id(user_id)
            .one(&db)
            .await
            .unwrap_or(None)
            .map(|u| UserResponse {
                user_id: u.id,
                username: u.username,
                email: u.email,
            });

        if let Some(user) = user {
            let user_value = serde_json::to_value(user).unwrap();
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

