use crate::{auth::claims::AuthUser, response::ApiResponse};
use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use common::format_validation_errors;
use db::models::{
    assignment,
    assignment_submission::{self, Column as GradeColumn, Entity as GradeEntity},
    module, user,
    user_module_role::{self, Column as RoleColumn, Role},
};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, FromQueryResult, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait,
};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, FromQueryResult)]
pub struct GradeWithRelations {
    pub id: i64,
    pub earned: i64,
    pub total: i64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub user_id: i64,
    pub assignment_id: i64,
    pub assignment_name: String,
    pub module_id: i64,
    pub module_code: String,
    pub username: String,
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

    let mut query_builder = GradeEntity::find()
        .column_as(assignment_submission::Column::Id, "id")
        .column_as(assignment_submission::Column::Earned, "earned")
        .column_as(assignment_submission::Column::Total, "total")
        .column_as(assignment_submission::Column::CreatedAt, "created_at")
        .column_as(assignment_submission::Column::UpdatedAt, "updated_at")
        .column_as(assignment_submission::Column::UserId, "user_id")
        .column_as(assignment_submission::Column::AssignmentId, "assignment_id")
        .column_as(assignment::Column::Name, "assignment_name")
        .column_as(module::Column::Id, "module_id")
        .column_as(module::Column::Code, "module_code")
        .column_as(user::Column::Username, "username")
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
        .filter(user_module_role::Column::UserId.eq(caller_id));

    let role_to_check = query.role.unwrap_or(Role::Student);

    match role_to_check {
        Role::Student => {
            query_builder = query_builder.filter(GradeColumn::UserId.eq(caller_id));
        }
        Role::Lecturer | Role::AssistantLecturer | Role::Tutor => {
            query_builder = query_builder.filter(RoleColumn::Role.eq(role_to_check));
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

    query_builder = query_builder.filter(condition);

    if let Some(sort) = &query.sort {
        for s in sort.split(',') {
            let (field, order) = if s.starts_with('-') {
                (&s[1..], sea_orm::Order::Desc)
            } else {
                (s, sea_orm::Order::Asc)
            };

            match field {
                "score" => {
                    query_builder = query_builder.order_by(
                        sea_orm::prelude::Expr::cust(
                            "COALESCE((earned * 1.0) / NULLIF(total, 0), 0)",
                        ),
                        order,
                    );
                }
                "created_at" => {
                    query_builder = query_builder.order_by(GradeColumn::CreatedAt, order);
                }
                _ => {}
            }
        }
    } else {
        query_builder = query_builder.order_by(GradeColumn::CreatedAt, sea_orm::Order::Desc);
    }

    query_builder = query_builder.order_by(GradeColumn::Id, sea_orm::Order::Asc);

    let paginator = query_builder
        .into_model::<GradeWithRelations>()
        .paginate(db, per_page);

    let total = paginator.num_items().await.unwrap_or(0);
    let grades: Vec<GradeWithRelations> = paginator.fetch_page(page - 1).await.unwrap_or_default();

    let grades: Vec<GradeItem> = grades
        .into_iter()
        .map(|grade| {
            let percentage = if grade.total > 0 {
                (grade.earned as f64 / grade.total as f64) * 100.0
            } else {
                0.0
            };

            GradeItem {
                id: grade.id,
                score: Score {
                    earned: grade.earned,
                    total: grade.total,
                },
                percentage,
                created_at: grade.created_at.to_string(),
                updated_at: grade.updated_at.to_string(),
                module: ModuleInfo {
                    id: grade.module_id,
                    code: grade.module_code,
                },
                assignment: AssignmentInfo {
                    id: grade.assignment_id,
                    title: grade.assignment_name,
                },
                user: UserInfo {
                    id: grade.user_id,
                    username: grade.username,
                },
            }
        })
        .collect();

    if grades.is_empty() {
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
