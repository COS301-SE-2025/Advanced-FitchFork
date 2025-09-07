use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use common::format_validation_errors;
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, RelationTrait, QuerySelect, FromQueryResult, prelude::Expr,
};
use sea_orm_migration::prelude::Alias;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{auth::claims::AuthUser, response::ApiResponse};
use db::models::{
    assignment,
    assignment_submission::{self, Column as SubmissionColumn, Entity as SubmissionEntity},
    module,
    user,
    user_module_role::{self, Column as RoleColumn, Role},
};
use util::state::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct GetSubmissionsQuery {
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<u64>,
    pub query: Option<String>,
    pub role: Option<Role>,
    pub year: Option<i32>,
    pub is_late: Option<bool>,
    pub sort: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubmissionItem {
    pub id: i64,
    pub status: String,
    pub score: Score,
    pub created_at: String,
    pub updated_at: String,
    pub is_late: bool,
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
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct GetSubmissionsResponse {
    pub submissions: Vec<SubmissionItem>,
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
}

#[derive(Debug, FromQueryResult)]
pub struct SubmissionWithRelations {
    pub id: i64,
    pub earned: i64,
    pub total: i64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub user_id: i64,
    pub assignment_id: i64,
    pub assignment_name: String,
    pub assignment_description: Option<String>,
    pub assignment_due_date: chrono::NaiveDateTime,
    pub module_id: i64,
    pub module_code: String,
    pub username: String,
}

/// GET /api/me/submissions
///
/// Retrieves a paginated list of submissions relevant to the authenticated user.
/// The behavior of this endpoint changes based on the `role` query parameter,
/// allowing users to view submissions based on their permissions.
///
/// ### Authorization
/// Requires a valid bearer token.
///
/// ### Query Parameters
/// - `page` (optional, u64, min: 1): The page number for pagination. Defaults to 1.
/// - `per_page` (optional, u64, min: 1, max: 100): The number of items per page. Defaults to 20.
/// - `query` (optional, string): A search term to filter submissions by module code, student username, or assignment name.
/// - `role` (optional, string): The role to filter by. Can be `Student`, `Tutor`, `AssistantLecturer`, or `Lecturer`. Defaults to `Student`.
///   - If `Student`, returns only the authenticated user's submissions.
///   - If `Lecturer`, `Tutor`, etc., returns submissions for all students in modules where the user holds that role.
/// - `year` (optional, i32): Filters submissions by the module's academic year.
/// - `is_late` (optional, bool): Filters submissions based on whether they were submitted after the assignment's due date.
/// - `sort` (optional, string): A comma-separated list of fields to sort by. Prefix with `-` for descending order.
///   - Allowed fields: `score`, `created_at`.
///
/// ### Response: 200 OK
/// Returns a paginated list of submissions.
/// ```json
/// {
///   "success": true,
///   "message": "Submissions retrieved",
///   "data": {
///     "submissions": [
///       {
///         "id": 5512,
///         "status": "submitted",
///         "score": { "earned": 42, "total": 50 },
///         "created_at": "2025-08-08T12:15:00Z",
///         "updated_at": "2025-08-08T12:15:00Z",
///         "is_late": false,
///         "module": { "id": 344, "code": "COS344" },
///         "assignment": { "id": 102, "name": "A2 — Runtime & VM", "description": "..." },
///         "user": { "id": 2511, "username": "u23571561" }
///       }
///     ],
///     "page": 1,
///     "per_page": 20,
///     "total": 93
///   }
/// }
/// ```
///
/// ### Error Responses
/// - `400 Bad Request`: Invalid query parameters.
/// - `401 Unauthorized`: Not logged in.
pub async fn get_my_submissions(
    State(app_state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<GetSubmissionsQuery>,
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

    let mut query_builder = SubmissionEntity::find()
        .column_as(SubmissionColumn::Id, "id")
        .column_as(SubmissionColumn::Earned, "earned")
        .column_as(SubmissionColumn::Total, "total")
        .column_as(SubmissionColumn::CreatedAt, "created_at")
        .column_as(SubmissionColumn::UpdatedAt, "updated_at")
        .column_as(SubmissionColumn::UserId, "user_id")
        .column_as(SubmissionColumn::AssignmentId, "assignment_id")
        .column_as(assignment::Column::Name, "assignment_name")
        .column_as(assignment::Column::Description, "assignment_description")
        .column_as(assignment::Column::DueDate, "assignment_due_date")
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
            query_builder = query_builder.filter(SubmissionColumn::UserId.eq(caller_id));
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
                .add(module::Column::Code.like(&pattern))
                .add(user::Column::Username.like(&pattern))
                .add(assignment::Column::Name.like(&pattern)),
        );
    }

    if let Some(year) = query.year {
        condition = condition.add(module::Column::Year.eq(year));
    }

    if let Some(is_late) = query.is_late {
        let submission_alias = Alias::new("assignment_submissions");
        let assignment_alias = Alias::new("assignments");
        
        let created_at = Expr::col((submission_alias.clone(), SubmissionColumn::CreatedAt));
        let due_date = Expr::col((assignment_alias.clone(), assignment::Column::DueDate));

        if is_late {
            condition = condition.add(created_at.gt(due_date));
        } else {
            condition = condition.add(created_at.lte(due_date));
        }
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
                    query_builder = query_builder.order_by(SubmissionColumn::CreatedAt, order);
                }
                _ => {}
            }
        }
    } else {
        query_builder = query_builder.order_by(SubmissionColumn::CreatedAt, sea_orm::Order::Desc);
    }
    
    query_builder = query_builder.order_by(SubmissionColumn::Id, sea_orm::Order::Asc);

    let paginator = query_builder
        .into_model::<SubmissionWithRelations>()
        .paginate(db, per_page);
    
    let total = paginator.num_items().await.unwrap_or(0);
    let submissions_db: Vec<SubmissionWithRelations> = paginator
        .fetch_page(page - 1)
        .await
        .unwrap_or_default();

    let submissions: Vec<SubmissionItem> = submissions_db
        .into_iter()
        .map(|s| {
            let is_late = s.created_at > s.assignment_due_date;

            SubmissionItem {
                id: s.id,
                status: "submitted".to_string(),
                score: Score {
                    earned: s.earned,
                    total: s.total,
                },
                created_at: s.created_at.to_string(),
                updated_at: s.updated_at.to_string(),
                is_late,
                module: ModuleInfo {
                    id: s.module_id,
                    code: s.module_code,
                },
                assignment: AssignmentInfo {
                    id: s.assignment_id,
                    name: s.assignment_name,
                    description: s.assignment_description,
                },
                user: UserInfo {
                    id: s.user_id,
                    username: s.username,
                },
            }
        })
        .collect();

    if submissions.is_empty() {
        return (
            StatusCode::OK,
            Json(ApiResponse::success(
                GetSubmissionsResponse {
                    submissions: vec![],
                    page,
                    per_page,
                    total: 0,
                },
                "No submissions found",
            )),
        ).into_response();
    }

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            GetSubmissionsResponse {
                submissions,
                page,
                per_page,
                total,
            },
            "Submissions retrieved",
        )),
    )
        .into_response()
}