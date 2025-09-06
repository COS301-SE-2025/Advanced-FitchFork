//! Assignment routes and response models.
//!
//! Provides endpoints and data structures for managing assignments within modules:
//!
//! - `GET /api/modules/{module_id}/assignments/{assignment_id}`  
//!   Retrieve a specific assignment along with its associated files.
//!
//! - `GET /api/modules/{module_id}/assignments`  
//!   Retrieve a paginated and optionally filtered list of assignments.
//!
//! - `GET /api/modules/{module_id}/assignments/{assignment_id}/stats`  
//!   Retrieve submission statistics for a specific assignment.
//!
//! - `GET /api/modules/{module_id}/assignments/{assignment_id}/readiness`  
//!   Retrieve a readiness report for a specific assignment, checking whether all required components are present.
//!
//! **Models:**  
//! - `AssignmentFileResponse`: Assignment data plus associated files.  
//! - `FilterReq` / `FilterResponse`: Query and response for paginated assignment lists.  
//! - `StatResponse` / `PerStudentSubmission`: Assignment submission statistics.  
//! - `AssignmentReadiness`: Detailed readiness report for an assignment.
//!
//! All endpoints use `AppState` for database access and return JSON-wrapped `ApiResponse`.

use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, sea_query::Expr,
};
use util::state::AppState;
use crate::{auth::AuthUser, response::ApiResponse};
use crate::routes::modules::assignments::common::{File, AssignmentResponse};
use db::{
    models::{
        assignment::{
            self, AssignmentType, Column as AssignmentColumn, Entity as AssignmentEntity, Model as AssignmentModel
        }, 
        assignment_file,
        assignment_submission, 
        user
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssignmentFileResponse {
    pub assignment: AssignmentResponse,
    pub files: Vec<File>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_mark: Option<BestMark>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BestMark {
    pub earned: i64,
    pub total: i64,
    pub attempt: i64,
    pub submission_id: i64,
}

impl From<AssignmentModel> for AssignmentFileResponse {
    fn from(assignment: AssignmentModel) -> Self {
        Self {
            assignment: AssignmentResponse {
                id: assignment.id,
                module_id: assignment.module_id as i64,
                name: assignment.name,
                description: assignment.description,
                status: assignment.status.to_string(),
                assignment_type: assignment.assignment_type.to_string(),
                available_from: assignment.available_from.to_rfc3339(),
                due_date: assignment.due_date.to_rfc3339(),
                created_at: assignment.created_at.to_rfc3339(),
                updated_at: assignment.updated_at.to_rfc3339(),
            },
            files: Vec::new(),
            best_mark: None,
        }
    }
}


/// GET /api/modules/{module_id}/assignments/{assignment_id}
///
/// Retrieve a specific assignment along with its associated files.  
/// Accessible to all users assigned to the module.  
/// If the authenticated user is a **student**, the response will also include
/// their current **best mark** for this assignment, based on the grading policy.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to retrieve
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "message": "Assignment retrieved successfully",
///   "data": {
///     "assignment": {
///       "id": 123,
///       "module_id": 456,
///       "name": "Assignment 1",
///       "description": "This is a sample assignment",
///       "assignment_type": "Assignment",
///       "status": "open",
///       "available_from": "2024-01-01T00:00:00Z",
///       "due_date": "2024-01-31T23:59:59Z",
///       "created_at": "2024-01-01T00:00:00Z",
///       "updated_at": "2024-01-15T12:00:00Z"
///     },
///     "files": [
///       {
///         "id": "789",
///         "filename": "assignment.pdf",
///         "path": "module_456/assignment_123/assignment.pdf",
///         "file_type": "config",
///         "created_at": "2024-01-01T00:00:00Z",
///         "updated_at": "2024-01-01T00:00:00Z"
///       }
///     ],
///     "best_mark": {
///       "earned": 85,
///       "total": 100,
///       "attempt": 2,
///       "submission_id": 4567
///     }
///   }
/// }
/// ```
///
/// - `404 Not Found`  
/// Assignment does not exist in this module.
/// ```json
/// {
///   "success": false,
///   "message": "Assignment not found"
/// }
/// ```
///
/// - `500 Internal Server Error`  
/// Returned if the database or file fetch fails.
/// ```json
/// {
///   "success": false,
///   "message": "Failed to retrieve files: <error details>"
/// }
/// ```
pub async fn get_assignment(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    let db = app_state.db();

    let assignment_res = assignment::Entity::find()
        .filter(assignment::Column::Id.eq(assignment_id as i32))
        .filter(assignment::Column::ModuleId.eq(module_id as i32))
        .one(db)
        .await;

    match assignment_res {
        Ok(Some(a)) => {
            let files_res = assignment_file::Entity::find()
                .filter(assignment_file::Column::AssignmentId.eq(a.id))
                .all(db)
                .await;

            match files_res {
                Ok(files) => {
                    let converted_files: Vec<File> =
                        files.into_iter().map(File::from).collect();

                    let mut best_mark = None;

                    let user_id: i64 = user.sub;

                    // only if user is a student in this module
                    if user::Model::is_in_role(db, user_id, module_id, "student")
                        .await
                        .unwrap_or(false)
                    {
                        if let Ok(Some(sub)) =
                            assignment_submission::Model::get_best_for_user(db, &a, user_id).await
                        {
                            best_mark = Some(BestMark {
                                earned: sub.earned,
                                total: sub.total,
                                attempt: sub.attempt,
                                submission_id: sub.id,
                            });
                        }
                    }

                    let response = AssignmentFileResponse {
                        assignment: AssignmentResponse::from(a),
                        files: converted_files,
                        best_mark,
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
                    Json(ApiResponse::<AssignmentFileResponse>::error(&format!(
                        "Failed to retrieve files: {}",
                        e
                    ))),
                ),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<AssignmentFileResponse>::error(
                "Assignment not found",
            )),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignmentFileResponse>::error(&format!(
                "An error occurred: {}",
                e
            ))),
        ),
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
    pub assignments: Vec<AssignmentResponse>,
    pub page: i32,
    pub per_page: i32,
    pub total: i32,
}

impl FilterResponse {
    fn new(
        assignments: Vec<AssignmentResponse>,
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

/// GET /api/modules/{module_id}/assignments
///
/// Retrieve a paginated and optionally filtered list of assignments for a module. Accessible to users assigned to the module.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module to retrieve assignments from
///
/// ### Query Parameters
/// - `page` (optional, i32): Page number for pagination. Defaults to 1, minimum value is 1
/// - `per_page` (optional, i32): Number of items per page. Defaults to 20, maximum is 100, minimum is 1
/// - `sort` (optional, string): Comma-separated list of fields to sort by. Prefix with `-` for descending order (e.g., `-due_date`)
/// - `query` (optional, string): Case-insensitive substring match applied to both `name` and `description`
/// - `name` (optional, string): Case-insensitive filter to match assignment names
/// - `assignment_type` (optional, string): Filter by assignment type ("Assignment" or "Practical")
/// - `available_before` (optional, string): Filter assignments available before this date/time (ISO 8601)
/// - `available_after` (optional, string): Filter assignments available after this date/time (ISO 8601)
/// - `due_before` (optional, string): Filter assignments due before this date/time (ISO 8601)
/// - `due_after` (optional, string): Filter assignments due after this date/time (ISO 8601)
///
/// **Allowed sort fields:** `name`, `description`, `due_date`, `available_from`, `assignment_type`, `created_at`, `updated_at`
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "message": "Assignments retrieved successfully",
///   "data": {
///     "assignments": [
///       {
///         "id": 123,
///         "module_id": 456,
///         "name": "Assignment 1",
///         "description": "This is a sample assignment",
///         "assignment_type": "Assignment",
///         "available_from": "2024-01-01T00:00:00Z",
///         "due_date": "2024-01-31T23:59:59Z",
///         "created_at": "2024-01-01T00:00:00Z",
///         "updated_at": "2024-01-01T00:00:00Z"
///       }
///     ],
///     "page": 1,
///     "per_page": 20,
///     "total": 1
///   }
/// }
/// ```
///
/// - `400 Bad Request`
/// ```json
/// {
///   "success": false,
///   "message": "Invalid field used" // or "Invalid assignment_type"
/// }
/// ```
///
/// - `500 Internal Server Error`
/// ```json
/// {
///   "success": false,
///   "message": "<database error details>"
/// }
/// ```
pub async fn get_assignments(
    State(app_state): State<AppState>,
    Path(module_id): Path<i64>,
    Query(params): Query<FilterReq>,
) -> impl IntoResponse {
    let db = app_state.db();
    
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100).max(1);

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
            let field = field.trim().trim_start_matches('-');
            if !valid_fields.contains(&field) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<FilterResponse>::error("Invalid field used")),
                );
            }
        }
    }

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

    let mut query = AssignmentEntity::find().filter(condition);

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

    let paginator = query.clone().paginate(db, per_page as u64);
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
            let assignments: Vec<AssignmentResponse> = results
                .into_iter()
                .map(AssignmentResponse::from)
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
pub struct PerStudentSubmission {
    pub user_id: i64,
    pub username: String,
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

pub fn is_late(submission: DateTime<Utc>, due_date: DateTime<Utc>) -> bool {
    submission > due_date
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/stats
///
/// Retrieve submission statistics for a specific assignment. Only accessible by lecturers assigned to the module.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to get statistics for
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "message": "Stats retrieved successfully",
///   "data": {
///     "assignment_id": 123,
///     "total_submissions": 15,
///     "unique_submitters": 12,
///     "late_submissions": 3,
///     "per_student_submission_count": [
///       {
///         "user_id": 456,
///         "username": "john.doe",
///         "count": 2,
///         "latest_at": "2024-01-31T23:59:59Z",
///         "latest_late": false
///       },
///       {
///         "user_id": 789,
///         "username": "jane.smith",
///         "count": 1,
///         "latest_at": "2024-02-01T01:30:00Z",
///         "latest_late": true
///       }
///     ]
///   }
/// }
/// ```
///
/// - `404 Not Found`
/// ```json
/// {
///   "success": false,
///   "message": "Assignment not found"
/// }
/// ```
///
/// - `500 Internal Server Error`
/// ```json
/// {
///   "success": false,
///   "message": "Database error" // or "Failed to fetch student numbers"
/// }
/// ```
pub async fn get_assignment_stats(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>
) -> impl IntoResponse {
    let db = app_state.db();

    let assignment = match AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id as i32))
        .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
        .one(db)
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

    match assignment_submission::Entity::find()
        .filter(assignment_submission::Column::AssignmentId.eq(assignment_id as i32))
        .order_by_desc(assignment_submission::Column::CreatedAt)
        .all(db)
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
                .all(db)
                .await;

            let mut user_id_to_username = HashMap::new();
            match user_models {
                Ok(users) => {
                    for user in users {
                        user_id_to_username.insert(user.id, user.username);
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
                let username = user_id_to_username
                    .get(user_id)
                    .cloned()
                    .unwrap_or_else(|| "UNKNOWN".to_string());

                per_student_submission_count.push(PerStudentSubmission {
                    user_id: *user_id,
                    username,
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

#[derive(Debug, Serialize)]
pub struct AssignmentReadiness {
    pub config_present: bool,
    pub tasks_present: bool,
    pub main_present: bool,
    pub memo_present: bool,
    pub makefile_present: bool,
    pub memo_output_present: bool,
    pub mark_allocator_present: bool,
    pub is_ready: bool,
}

/// GET /api/modules/:module_id/assignments/:assignment_id/readiness
///
/// Retrieve a detailed readiness report for a specific assignment.
/// The report includes boolean flags indicating whether each required
/// component of the assignment is present on disk or in the database.
///
/// This endpoint is useful to check if an assignment is fully set up
/// and eligible to transition from `Setup` to `Ready` state.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment.
/// - `assignment_id` (i64): The ID of the assignment to check readiness for.
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "message": "Assignment readiness checked successfully",
///   "data": {
///     "config_present": true,
///     "tasks_present": true,
///     "main_present": true,
///     "memo_present": true,
///     "makefile_present": true,
///     "memo_output_present": true,
///     "mark_allocator_present": true,
///     "is_ready": true
///   }
/// }
/// ```
///
/// - `500 Internal Server Error`
/// ```json
/// {
///   "success": false,
///   "message": "Failed to compute readiness: <error details>"
/// }
/// ```
///
pub async fn get_assignment_readiness(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> (StatusCode, Json<ApiResponse<AssignmentReadiness>>) {
    let db = app_state.db();

    match AssignmentModel::compute_readiness_report(db, module_id, assignment_id).await {
        Ok(report) => {
            if report.is_ready() {
                if let Err(e) =
                    AssignmentModel::try_transition_to_ready(db, module_id, assignment_id).await
                {
                    tracing::warn!(
                        "Failed to transition assignment {} to Ready: {:?}",
                        assignment_id,
                        e
                    );
                }
            }

            let response = AssignmentReadiness {
                config_present: report.config_present,
                tasks_present: report.tasks_present,
                main_present: report.main_present,
                memo_present: report.memo_present,
                makefile_present: report.makefile_present,
                memo_output_present: report.memo_output_present,
                mark_allocator_present: report.mark_allocator_present,
                is_ready: report.is_ready(),
            };

            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "Assignment readiness checked successfully",
                )),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignmentReadiness>::error(&format!(
                "Failed to compute readiness: {}",
                e
            ))),
        ),
    }
}