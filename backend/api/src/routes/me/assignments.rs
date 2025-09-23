//! # My Assignments Handlers
//!
//! Provides endpoints to fetch assignments for the currently authenticated user.
//!
//! Users can retrieve a paginated list of assignments filtered by role, year, status,
//! search query, and sorted by various fields. Only assignments in modules the user
//! is associated with are returned.

use crate::{auth::AuthUser, response::ApiResponse};
use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use db::grade::{
    GradeComputationError, GradeComputationOptions, compute_assignment_grade_for_student,
    compute_assignment_grades,
};
use db::models::{
    assignment::{self, Status as AssignmentStatus},
    module,
    user_module_role::{self, Role as ModuleRole},
};
use migration::Expr;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, JoinType, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use util::state::AppState;

/// Query parameters for filtering, sorting, and pagination of assignments
#[derive(Debug, Deserialize)]
pub struct AssignmentFilterReq {
    /// Page number (default: 1)
    pub page: Option<i32>,
    /// Items per page (default: 20)
    pub per_page: Option<i32>,
    /// Search query (matches assignment title or module code)
    pub query: Option<String>,
    /// Filter assignments by role (lecturer, assistant_lecturer, tutor, student)
    pub role: Option<String>,
    /// Filter by module year
    pub year: Option<i32>,
    /// Filter by assignment status
    pub status: Option<String>,
    /// Sort fields (comma-separated, prefix with `-` for descending)
    pub sort: Option<String>,
    /// Filter assignments for a specific module
    pub module_id: Option<i64>,
}

/// Response object for a module
#[derive(Serialize)]
pub struct ModuleResponse {
    pub id: i64,
    pub code: String,
}

/// Response object for an assignment
#[derive(Serialize)]
pub struct AssignmentResponse {
    pub id: i64,
    pub name: String,
    pub status: String,
    pub available_from: String,
    pub due_date: String,
    pub created_at: String,
    pub updated_at: String,
    pub module: ModuleResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grade: Option<AssignmentGrade>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submission_summary: Option<AssignmentSubmissionSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readiness: Option<assignment::ReadinessReport>,
}

/// Response for a paginated list of assignments
#[derive(Serialize)]
pub struct FilterAssignmentResponse {
    pub assignments: Vec<AssignmentResponse>,
    pub page: i32,
    pub per_page: i32,
    pub total: i32,
}

#[derive(Serialize)]
pub struct AssignmentGrade {
    pub percentage: f32,
    pub earned: i64,
    pub total: i64,
}

#[derive(Serialize)]
pub struct AssignmentSubmissionSummary {
    pub submitted: u32,
    pub total_students: u32,
}

impl FilterAssignmentResponse {
    fn new(assignments: Vec<AssignmentResponse>, page: i32, per_page: i32, total: i32) -> Self {
        Self {
            assignments,
            page,
            per_page,
            total,
        }
    }
}

/// Retrieves assignments for the currently authenticated user.
///
/// **Endpoint:** `GET /my/assignments`  
/// **Permissions:** User must be associated with at least one module (student, tutor, lecturer, assistant)
///
/// ### Query parameters
/// - `page` → Page number (default: 1)
/// - `per_page` → Number of items per page (default: 20, max: 100)
/// - `query` → Search query in assignment title or module code
/// - `role` → Filter assignments by user role
/// - `year` → Filter assignments by module year
/// - `status` → Filter assignments by assignment status
/// - `sort` → Sort assignments by fields (e.g., `due_date,-available_from`)
///
/// ### Responses
/// - `200 OK` → Assignments retrieved successfully
/// ```json
/// {
///   "success": true,
///   "data": {
///     "assignments": [ /* Assignment objects */ ],
///     "page": 1,
///     "per_page": 20,
///     "total": 42
///   },
///   "message": "Assignments retrieved"
/// }
/// ```
/// - `500 Internal Server Error` → Failed to retrieve assignments
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Failed to retrieve assignments"
/// }
/// ```
pub async fn get_my_assignments(
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Query(params): Query<AssignmentFilterReq>,
) -> impl IntoResponse {
    let user_id = claims.sub;
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);

    let allowed_roles = vec!["lecturer", "assistant_lecturer", "tutor", "student"];
    let requested_role = params
        .role
        .clone()
        .filter(|r| allowed_roles.contains(&r.as_str()));

    let memberships = user_module_role::Entity::find()
        .filter(user_module_role::Column::UserId.eq(user_id))
        .filter(user_module_role::Column::Role.is_in(allowed_roles.clone()))
        .all(db)
        .await
        .unwrap_or_default();

    if memberships.is_empty() {
        let response = FilterAssignmentResponse::new(vec![], page, per_page, 0);
        return (
            StatusCode::OK,
            Json(ApiResponse::success(response, "Assignments retrieved")),
        )
            .into_response();
    }

    let module_ids: Vec<i64> = memberships
        .iter()
        .filter(|m| {
            if let Some(module_filter) = params.module_id {
                if m.module_id != module_filter {
                    return false;
                }
            }

            requested_role
                .as_ref()
                .map_or(true, |r| &m.role.to_string() == r)
        })
        .map(|m| m.module_id)
        .collect();

    if module_ids.is_empty() {
        let response = FilterAssignmentResponse::new(vec![], page, per_page, 0);
        return (
            StatusCode::OK,
            Json(ApiResponse::success(response, "Assignments retrieved")),
        )
            .into_response();
    }

    let mut module_role_map: HashMap<i64, Vec<user_module_role::Role>> = HashMap::new();
    for membership in &memberships {
        module_role_map
            .entry(membership.module_id)
            .or_default()
            .push(membership.role.clone());
    }

    let mut condition = Condition::all().add(assignment::Column::ModuleId.is_in(module_ids));

    if let Some(year) = params.year {
        condition = condition.add(Expr::col((module::Entity, module::Column::Year)).eq(year));
    }

    if let Some(ref status) = params.status {
        match AssignmentStatus::from_str(status) {
            Ok(parsed) => {
                condition = condition.add(assignment::Column::Status.eq(parsed));
            }
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<FilterAssignmentResponse>::error(
                        "Invalid status parameter",
                    )),
                )
                    .into_response();
            }
        }
    }

    if let Some(ref q) = params.query {
        let pattern = format!("%{}%", q.to_lowercase());
        condition = condition.add(
            Condition::any()
                .add(Expr::cust("LOWER(assignment.title)").like(&pattern))
                .add(Expr::cust("LOWER(module.code)").like(&pattern)),
        );
    }

    let mut query = assignment::Entity::find()
        .join(JoinType::InnerJoin, assignment::Relation::Module.def())
        .filter(condition);

    if let Some(sort_param) = &params.sort {
        for sort in sort_param.split(',') {
            let (field, asc) = if sort.starts_with('-') {
                (&sort[1..], false)
            } else {
                (sort, true)
            };
            query = match field {
                "due_date" => {
                    if asc {
                        query.order_by_asc(assignment::Column::DueDate)
                    } else {
                        query.order_by_desc(assignment::Column::DueDate)
                    }
                }
                "available_from" => {
                    if asc {
                        query.order_by_asc(assignment::Column::AvailableFrom)
                    } else {
                        query.order_by_desc(assignment::Column::AvailableFrom)
                    }
                }
                _ => query,
            };
        }
    } else {
        query = query
            .order_by_asc(assignment::Column::DueDate)
            .order_by_asc(assignment::Column::Id);
    }

    let paginator = query.clone().paginate(db, per_page as u64);
    let total = match paginator.num_items().await {
        Ok(n) => n as i32,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<FilterAssignmentResponse>::error(
                    "Error counting assignments",
                )),
            )
                .into_response();
        }
    };

    match paginator.fetch_page((page - 1) as u64).await {
        Ok(results) => {
            let mut assignments_vec = Vec::new();

            let mut module_student_count_cache: HashMap<i64, u32> = HashMap::new();
            let mut assignment_submitted_cache: HashMap<i64, u32> = HashMap::new();

            for a in results {
                let m = module::Entity::find_by_id(a.module_id)
                    .one(db)
                    .await
                    .unwrap_or(None);
                if m.is_none() {
                    continue;
                }
                let m = m.unwrap();

                let roles = module_role_map.get(&m.id).cloned().unwrap_or_default();
                let is_student = roles.iter().any(|r| matches!(r, ModuleRole::Student));
                let is_staff = roles.iter().any(|r| {
                    matches!(
                        r,
                        ModuleRole::Lecturer | ModuleRole::AssistantLecturer | ModuleRole::Tutor
                    )
                });
                let is_lecturer_or_assistant = roles
                    .iter()
                    .any(|r| matches!(r, ModuleRole::Lecturer | ModuleRole::AssistantLecturer));

                let mut grade: Option<AssignmentGrade> = None;
                if is_student {
                    match compute_assignment_grade_for_student(db, m.id, a.id, user_id).await {
                        Ok(Some(selection)) => {
                            grade = Some(AssignmentGrade {
                                percentage: selection.score_pct,
                                earned: selection.submission.earned,
                                total: selection.submission.total,
                            });
                        }
                        Ok(None) => {}
                        Err(GradeComputationError::AssignmentNotFound) => {
                            eprintln!(
                                "get_my_assignments: assignment {} not found in module {}",
                                a.id, m.id
                            );
                        }
                        Err(GradeComputationError::ExecutionConfig(err)) => {
                            eprintln!(
                                "get_my_assignments: execution config error for assignment {}: {}",
                                a.id, err
                            );
                        }
                        Err(GradeComputationError::Database(err)) => {
                            eprintln!(
                                "get_my_assignments: database error computing grade for assignment {}: {}",
                                a.id, err
                            );
                        }
                    }
                }

                let mut submission_summary: Option<AssignmentSubmissionSummary> = None;
                if is_staff {
                    let total_students = if let Some(count) = module_student_count_cache.get(&m.id)
                    {
                        *count
                    } else {
                        let count = match user_module_role::Entity::find()
                            .filter(user_module_role::Column::ModuleId.eq(m.id))
                            .filter(
                                user_module_role::Column::Role.eq(user_module_role::Role::Student),
                            )
                            .count(db)
                            .await
                        {
                            Ok(n) => n as u32,
                            Err(err) => {
                                eprintln!(
                                    "get_my_assignments: failed to count students for module {}: {}",
                                    m.id, err
                                );
                                0
                            }
                        };
                        module_student_count_cache.insert(m.id, count);
                        count
                    };

                    let submitted = if let Some(count) = assignment_submitted_cache.get(&a.id) {
                        *count
                    } else {
                        let count = match compute_assignment_grades(
                            db,
                            m.id,
                            a.id,
                            GradeComputationOptions::default(),
                        )
                        .await
                        {
                            Ok(res) => res.grades.len() as u32,
                            Err(GradeComputationError::AssignmentNotFound) => {
                                eprintln!(
                                    "get_my_assignments: assignment {} not found while computing submissions",
                                    a.id
                                );
                                0
                            }
                            Err(GradeComputationError::ExecutionConfig(err)) => {
                                eprintln!(
                                    "get_my_assignments: execution config error while computing submissions for assignment {}: {}",
                                    a.id, err
                                );
                                0
                            }
                            Err(GradeComputationError::Database(err)) => {
                                eprintln!(
                                    "get_my_assignments: database error while computing submissions for assignment {}: {}",
                                    a.id, err
                                );
                                0
                            }
                        };
                        assignment_submitted_cache.insert(a.id, count);
                        count
                    };

                    submission_summary = Some(AssignmentSubmissionSummary {
                        submitted,
                        total_students,
                    });
                }

                let readiness = if is_lecturer_or_assistant {
                    match assignment::Model::compute_readiness_report(db, m.id, a.id).await {
                        Ok(report) => Some(report),
                        Err(err) => {
                            eprintln!(
                                "get_my_assignments: readiness compute error for assignment {}: {}",
                                a.id, err
                            );
                            None
                        }
                    }
                } else {
                    None
                };

                assignments_vec.push(AssignmentResponse {
                    id: a.id,
                    name: a.name,
                    status: a.status.to_string(),
                    available_from: a.available_from.to_string(),
                    due_date: a.due_date.to_string(),
                    created_at: a.created_at.to_string(),
                    updated_at: a.updated_at.to_string(),
                    module: ModuleResponse {
                        id: m.id,
                        code: m.code,
                    },
                    grade,
                    submission_summary,
                    readiness,
                });
            }

            let response = FilterAssignmentResponse::new(assignments_vec, page, per_page, total);
            (
                StatusCode::OK,
                Json(ApiResponse::success(response, "Assignments retrieved")),
            )
                .into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<FilterAssignmentResponse>::error(
                "Failed to retrieve assignments",
            )),
        )
            .into_response(),
    }
}
