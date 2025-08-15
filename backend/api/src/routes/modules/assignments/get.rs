use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sea_orm::{
    ColumnTrait, /*Condition,*/ EntityTrait, /*PaginatorTrait,*/ QueryFilter,
    QueryOrder, /*sea_query::Expr,*/
};
use crate::response::ApiResponse;
use crate::routes::modules::assignments::common::{File, AssignmentResponse};
use db::{
    models::{
        assignment::{
            self, /*AssignmentType,*/ Column as AssignmentColumn, Entity as AssignmentEntity, Model as AssignmentModel
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
        }
    }
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}
///
/// Retrieve a specific assignment along with its associated files. Accessible to users assigned to the module.
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
///       "available_from": "2024-01-01T00:00:00Z",
///       "due_date": "2024-01-31T23:59:59Z",
///       "created_at": "2024-01-01T00:00:00Z",
///       "updated_at": "2024-01-01T00:00:00Z"
///     },
///     "files": [
///       {
///         "id": "789",
///         "filename": "assignment.pdf",
///         "path": "module_456/assignment_123/assignment.pdf",
///         "created_at": "2024-01-01T00:00:00Z",
///         "updated_at": "2024-01-01T00:00:00Z"
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
///   "message": "Failed to retrieve files: <error details>" // or "An error occurred: <error details>"
/// }
/// ```
pub async fn get_assignment(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let db = db::get_connection().await;

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
                    let converted_files: Vec<File> = files
                        .into_iter()
                        .map(|f| File {
                            id: f.id.to_string(),
                            filename: f.filename,
                            path: f.path,
                            file_type: f.file_type.to_string(),
                            created_at: f.created_at.to_rfc3339(),
                            updated_at: f.updated_at.to_rfc3339(),
                        })
                        .collect();

                    let response = AssignmentFileResponse {
                        assignment: AssignmentResponse::from(a),
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

// #[derive(Serialize)]
// pub struct FilterResponse {
//     pub assignments: Vec<AssignmentResponse>,
//     pub page: i32,
//     pub per_page: i32,
//     pub total: i32,
// }

// impl FilterResponse {
//     fn new(
//         assignments: Vec<AssignmentResponse>,
//         page: i32,
//         per_page: i32,
//         total: i32,
//     ) -> Self {
//         Self {
//             assignments,
//             page,
//             per_page,
//             total,
//         }
//     }
// }

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
/// **Allowed sort fields:** `name`, `description`, `assignment_type`, `status`, `available_from`, `due_date`, `created_at`, `updated_at`
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
    Path(_module_id): Path<i64>,
    Query(_params): Query<FilterReq>,
) -> impl IntoResponse {
    // let page = params.page.unwrap_or(1).max(1) as u64;
    // let per_page = (params.per_page.unwrap_or(20).min(100).max(1)) as u64;

    // // Validate sort fields (keeps existing behavior)
    // if let Some(ref sort_field) = params.sort {
    //     let valid_fields = [
    //         "name",
    //         "description", 
    //         "assignment_type",
    //         "status",
    //         "available_from",
    //         "due_date",
    //         "created_at",
    //         "updated_at",
    //     ];

    //     let mut valid = true;
    //     for field in sort_field.split(',') {
    //         let field = field.trim().trim_start_matches('-');
    //         if !valid_fields.contains(&field) {
    //             valid = false;
    //             break;
    //         }
    //     }

    //     if !valid {
    //         return (
    //             StatusCode::BAD_REQUEST,
    //             Json(ApiResponse::<FilterResponse>::error("Invalid field used")),
    //         );
    //     }
    // }

    // // Clone sort out before moving `params` into service
    // let sort_by = params.sort.clone();

    // // Move the whole `params` into the service. Service will build AssignmentFilter.
    // match assignment_service
    //     .filter_with_req(params, module_id, page, per_page, sort_by)
    //     .await
    // {
    //     Ok(result) => {
    //         let assignments: Vec<AssignmentResponse> = result
    //             .assignments
    //             .into_iter()
    //             .map(AssignmentResponse::from)
    //             .collect();

    //         let response = FilterResponse::new(
    //             assignments,
    //             page as i32,
    //             per_page as i32,
    //             result.total as i32,
    //         );

    //         (
    //             StatusCode::OK,
    //             Json(ApiResponse::success(
    //                 response,
    //                 "Assignments retrieved successfully",
    //             )),
    //         )
    //     }

    //     Err(err) => {
    //         // Treat custom DB errors (we use them for validation failures) as 400.
    //         match err {
    //             sea_orm::DbErr::Custom(msg) => (
    //                 StatusCode::BAD_REQUEST,
    //                 Json(ApiResponse::<FilterResponse>::error(&msg)),
    //             ),
    //             other => {
    //                 eprintln!("Service error: {:?}", other);
    //                 (
    //                     StatusCode::INTERNAL_SERVER_ERROR,
    //                     Json(ApiResponse::<FilterResponse>::error(
    //                         "Failed to retrieve assignments",
    //                     )),
    //                 )
    //             }
    //         }
    //     }
    // }
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
    Path((module_id, assignment_id)): Path<(i64, i64)>
) -> impl IntoResponse {
    let db = db::get_connection().await;

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
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> (StatusCode, Json<ApiResponse<AssignmentReadiness>>) {
    match AssignmentModel::compute_readiness_report(module_id, assignment_id).await {
        Ok(report) => {
            if report.is_ready() {
                if let Err(e) =
                    AssignmentModel::try_transition_to_ready(module_id, assignment_id).await
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