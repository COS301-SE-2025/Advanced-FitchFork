use crate::{auth::claims::AuthUser, response::ApiResponse};
use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use common::format_validation_errors;
use db::grade::{GradeComputationError, compute_assignment_grade_for_student};
use db::models::{
    assignment,
    assignment_submission::{self, Column as GradeColumn, Entity as GradeEntity},
    module, user,
    user_module_role::{self, Column as RoleColumn, Role},
};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, FromQueryResult, QueryFilter, QuerySelect, RelationTrait,
};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashSet};
use util::state::AppState;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct GetGradesQuery {
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<u64>,
    pub query: Option<String>,
    pub role: Option<Role>,
    pub year: Option<i32>,
    pub sort: Option<String>,
    pub module_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct GradeItem {
    pub id: i64,
    pub score: Score,
    pub percentage: f64,
    pub created_at: String,
    pub updated_at: String,
    pub module: ModuleInfo,
    pub assignment: AssignmentInfo,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct Score {
    pub earned: i64,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct ModuleInfo {
    pub id: i64,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct AssignmentInfo {
    pub id: i64,
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct GetGradesResponse {
    pub grades: Vec<GradeItem>,
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
}

#[derive(Debug, FromQueryResult, Clone)]
struct GradeScopeRow {
    pub assignment_id: i64,
    pub assignment_name: String,
    pub module_id: i64,
    pub module_code: String,
    pub user_id: i64,
}

/// GET /api/me/grades
///
/// Retrieves a paginated list of grades for the authenticated user.
/// The behavior of this endpoint changes based on the `role` query parameter,
/// allowing users to view grades based on their permissions.
///
/// ### Authorization
/// Requires a valid bearer token.
///
/// ### Query Parameters
/// - `page` (optional, u64, min: 1): The page number for pagination. Defaults to 1.
/// - `per_page` (optional, u64, min: 1, max: 100): The number of items per page. Defaults to 20.
/// - `query` (optional, string): A search term to filter grades by assignment title, student username, or module code.
/// - `role` (optional, string): The role to filter by. Can be `Student`, `Tutor`, `AssistantLecturer`, or `Lecturer`. Defaults to `Student`.
///   - If `Student`, returns only the authenticated user's grades.
///   - If `Lecturer`, `Tutor`, etc., returns grades for all students in modules where the user holds that role.
/// - `year` (optional, i32): Filters grades by the module's academic year.
/// - `sort` (optional, string): A comma-separated list of fields to sort by. Prefix with `-` for descending order.
///   - Allowed fields: `score`, `created_at`.
///
/// ### Response: 200 OK
/// Returns a paginated list of grades.
/// ```json
/// {
///   "success": true,
///   "message": "Grades retrieved successfully",
///   "data": {
///     "grades": [
///       {
///         "id": 1,
///         "score": {
///           "earned": 85,
///           "total": 100
///         },
///         "percentage": 85.0,
///         "created_at": "2025-08-17T10:00:00",
///         "updated_at": "2025-08-17T11:30:00",
///         "module": {
///           "id": 101,
///           "code": "CS101"
///         },
///         "assignment": {
///           "id": 201,
///           "title": "Introduction to Programming"
///         },
///         "user": {
///           "id": 42,
///           "username": "student_user"
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
/// ### Error Responses
/// - `400 Bad Request`: Invalid query parameters (e.g., `page` out of range).
/// - `403 Forbidden`: Missing or invalid authentication token.
/// - `500 Internal Server Error`: Database or other internal errors.
pub async fn get_my_grades(
    State(app_state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<GetGradesQuery>,
) -> impl IntoResponse {
    if let Err(e) = query.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(format_validation_errors(&e))),
        )
            .into_response();
    }

    let db = app_state.db();
    let caller_id = user.0.sub;

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    let role_to_check = query.role.unwrap_or(Role::Student);

    let mut query_builder = GradeEntity::find()
        .select_only()
        .column(GradeColumn::AssignmentId)
        .column_as(assignment::Column::Name, "assignment_name")
        .column_as(module::Column::Id, "module_id")
        .column_as(module::Column::Code, "module_code")
        .column_as(GradeColumn::UserId, "user_id")
        .join(
            sea_orm::JoinType::InnerJoin,
            assignment_submission::Relation::Assignment.def(),
        )
        .join(
            sea_orm::JoinType::InnerJoin,
            assignment::Relation::Module.def(),
        )
        .join(
            sea_orm::JoinType::InnerJoin,
            assignment_submission::Relation::User.def(),
        )
        .join(
            sea_orm::JoinType::InnerJoin,
            module::Relation::UserModuleRole.def(),
        )
        .filter(user_module_role::Column::UserId.eq(caller_id))
        .filter(assignment_submission::Column::IsPractice.eq(false))
        .filter(assignment_submission::Column::Ignored.eq(false))
        .distinct();

    match role_to_check {
        Role::Student => {
            query_builder = query_builder.filter(GradeColumn::UserId.eq(caller_id));
        }
        role => {
            query_builder = query_builder.filter(RoleColumn::Role.eq(role));
        }
    }

    let mut condition = Condition::all();

    if let Some(q) = &query.query {
        let pattern = format!("%{}%", q.to_lowercase());
        condition = condition.add(
            Condition::any()
                .add(assignment::Column::Name.like(&pattern))
                .add(user::Column::Username.like(&pattern))
                .add(module::Column::Code.like(&pattern)),
        );
    }

    if let Some(year) = query.year {
        condition = condition.add(module::Column::Year.eq(year));
    }

    if let Some(module_filter) = query.module_id {
        condition = condition.add(module::Column::Id.eq(module_filter));
    }

    query_builder = query_builder.filter(condition);

    let rows: Vec<GradeScopeRow> = match query_builder.into_model::<GradeScopeRow>().all(db).await {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("get_my_grades: query error: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to fetch grades")),
            )
                .into_response();
        }
    };

    let mut seen: HashSet<(i64, i64)> = HashSet::new();
    let mut scoped: Vec<GradeScopeRow> = Vec::new();
    for row in rows {
        if seen.insert((row.assignment_id, row.user_id)) {
            scoped.push(row);
        }
    }

    struct GradeWithMeta {
        item: GradeItem,
        percentage: f64,
        created_at: DateTime<Utc>,
    }

    let mut grade_rows: Vec<GradeWithMeta> = Vec::with_capacity(scoped.len());

    for row in scoped {
        match compute_assignment_grade_for_student(
            db,
            row.module_id,
            row.assignment_id,
            row.user_id,
        )
        .await
        {
            Ok(Some(selection)) => {
                let created_at_dt = selection.submission.created_at;
                let updated_at_dt = selection.submission.updated_at;
                let percentage = selection.score_pct as f64;

                let item = GradeItem {
                    id: selection.submission.id,
                    score: Score {
                        earned: selection.submission.earned,
                        total: selection.submission.total,
                    },
                    percentage,
                    created_at: created_at_dt.naive_utc().to_string(),
                    updated_at: updated_at_dt.naive_utc().to_string(),
                    module: ModuleInfo {
                        id: row.module_id,
                        code: row.module_code.clone(),
                    },
                    assignment: AssignmentInfo {
                        id: row.assignment_id,
                        title: row.assignment_name.clone(),
                    },
                    user: UserInfo {
                        id: selection.user.id,
                        username: selection.user.username.clone(),
                    },
                };

                grade_rows.push(GradeWithMeta {
                    percentage,
                    created_at: created_at_dt,
                    item,
                });
            }
            Ok(None) => continue,
            Err(GradeComputationError::AssignmentNotFound) => continue,
            Err(GradeComputationError::ExecutionConfig(e)) => {
                eprintln!("get_my_grades: execution config error: {e}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Failed to compute grades")),
                )
                    .into_response();
            }
            Err(GradeComputationError::Database(e)) => {
                eprintln!("get_my_grades: compute error: {e}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Failed to compute grades")),
                )
                    .into_response();
            }
        }
    }

    if grade_rows.is_empty() {
        return (
            StatusCode::OK,
            Json(ApiResponse::success(
                GetGradesResponse {
                    grades: vec![],
                    page,
                    per_page,
                    total: 0,
                },
                "No grades found",
            )),
        )
            .into_response();
    }

    let mut sort_fields: Vec<(String, bool)> = Vec::new();
    if let Some(sort) = &query.sort {
        for raw in sort.split(',') {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                continue;
            }
            let (field, desc) = if let Some(rest) = trimmed.strip_prefix('-') {
                (rest.to_string(), true)
            } else {
                (trimmed.to_string(), false)
            };

            match field.as_str() {
                "score" | "created_at" => sort_fields.push((field, desc)),
                _ => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::<()>::error("Invalid sort field")),
                    )
                        .into_response();
                }
            }
        }
    } else {
        sort_fields.push(("created_at".to_string(), true));
    }

    for (field, desc) in sort_fields.iter().rev() {
        match field.as_str() {
            "score" => grade_rows.sort_by(|a, b| {
                let cmp = a
                    .percentage
                    .partial_cmp(&b.percentage)
                    .unwrap_or(Ordering::Equal);
                if *desc { cmp.reverse() } else { cmp }
            }),
            "created_at" => grade_rows.sort_by(|a, b| {
                let cmp = a.created_at.cmp(&b.created_at);
                if *desc { cmp.reverse() } else { cmp }
            }),
            _ => {}
        }
    }

    let total = grade_rows.len() as u64;
    let start = (page.saturating_sub(1) * per_page) as usize;

    let grades: Vec<GradeItem> = if start >= grade_rows.len() {
        Vec::new()
    } else {
        grade_rows
            .into_iter()
            .skip(start)
            .take(per_page as usize)
            .map(|meta| meta.item)
            .collect()
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            GetGradesResponse {
                grades,
                page,
                per_page,
                total,
            },
            "Grades retrieved successfully",
        )),
    )
        .into_response()
}
