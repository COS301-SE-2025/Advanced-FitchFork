use axum::{
    extract::{Path, Query, State, Extension},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, JoinType, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Order
};

use db::models::{
    user,
    user_module_role::{self, Column as RoleCol, Role},
    user::Model as UserModel
};
use util::state::AppState;
use crate::{
    auth::AuthUser,
    response::{ApiResponse},
    routes::modules::common::{RoleResponse, RoleQuery, PaginatedRoleResponse},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, serde::Deserialize)]
pub struct RoleParam {
    pub role: Role,
}

/// GET /api/modules/{module_id}/personnel
///
/// Retrieve a paginated list of users assigned to a specific module and role.
///
/// ### Query Parameters:
/// - `role` (required): The target role to query. Must be one of:
///   - `"lecturer"`
///   - `"tutor"`
///   - `"assistant_lecturer"`
///   - `"student"`
///
/// - `page` (optional): Page number to fetch (default: 1, min: 1)
/// - `per_page` (optional): Number of results per page (default: 20, max: 100)
/// - `query` (optional): Fuzzy match against username or email
/// - `email` (optional): Exact or partial email match (ignored if `query` is present)
/// - `username` (optional): Exact or partial username match (ignored if `query` is present)
/// - `sort` (optional): Field to sort by (ascending unless prefixed with `-`)
///   - `"email"`, `"-email"`
///   - `"username"`, `"-username"`
///   - `"created_at"`, `"-created_at"`
///
/// ### Access Control:
/// - Admins can retrieve any role for any module
/// - Non-admins must be assigned to the module
///   - To view `lecturers`, the requesting user must be a `lecturer` in that module
///   - To view other roles (`tutor`, `assistant_lecturer`, `student`), the user must have any role in the module
///
/// ### Response: 200 OK
/// ```json
/// {
///   "success": true,
///   "message": "Personnel retrieved successfully",
///   "data": {
///     "users": [
///       {
///         "id": 42,
///         "username": "jdoe",
///         "email": "jdoe@example.com",
///         ...
///       }
///     ],
///     "page": 1,
///     "per_page": 20,
///     "total": 68
///   }
/// }
/// ```
///
/// ### Errors:
///
/// - `403 Forbidden`: Not authorized to view users in this module or for this role
/// ```json
/// {
///   "success": false,
///   "message": "You do not have permission to view this module's users"
/// }
/// ```
///
/// - `500 Internal Server Error`: Unexpected database failure
/// ```json
/// {
///   "success": false,
///   "message": "Internal server error"
/// }
/// ```
pub async fn get_personnel(
    State(app_state): State<AppState>,
    Path(module_id): Path<i64>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Query(role_param): Query<RoleParam>,
    Query(params): Query<RoleQuery>,
) -> Response {
    let db = app_state.db();
    let user_id = claims.sub;
    let requested_role = &role_param.role;

    // === Permission Enforcement ===
    if !claims.admin {
        if requested_role == &Role::Lecturer {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<()>::error("Only admins can view lecturers")),
            )
            .into_response();
        }

        // For other roles, user must be part of the module
        let is_member = user_module_role::Entity::find()
            .filter(
                Condition::all()
                    .add(RoleCol::UserId.eq(user_id))
                    .add(RoleCol::ModuleId.eq(module_id)),
            )
            .one(db)
            .await
            .map(|opt| opt.is_some())
            .unwrap_or(false);

        if !is_member {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<()>::error("You do not have permission to view this module's users")),
            )
            .into_response();
        }
    }


    // === Data Query ===
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    let mut condition = Condition::all()
        .add(RoleCol::ModuleId.eq(module_id))
        .add(RoleCol::Role.eq(requested_role.clone()));

    if let Some(ref q) = params.query {
        let q_lower = q.to_lowercase();
        condition = condition.add(
            Condition::any()
                .add(user::Column::Email.contains(&q_lower))
                .add(user::Column::Username.contains(&q_lower)),
        );
    } else {
        if let Some(ref email) = params.email {
            condition = condition.add(user::Column::Email.contains(email));
        }
        if let Some(ref sn) = params.username {
            condition = condition.add(user::Column::Username.contains(sn));
        }
    }

    let mut query = user::Entity::find()
        .join(
            JoinType::InnerJoin,
            user::Entity::belongs_to(user_module_role::Entity)
                .from(user::Column::Id)
                .to(RoleCol::UserId)
                .into(),
        )
        .filter(condition);

    match params.sort.as_deref() {
        Some("-email") => query = query.order_by_desc(user::Column::Email),
        Some("email") => query = query.order_by_asc(user::Column::Email),
        Some("-username") => query = query.order_by_desc(user::Column::Username),
        Some("username") => query = query.order_by_asc(user::Column::Username),
        Some("-created_at") => query = query.order_by_desc(user::Column::CreatedAt),
        Some("created_at") => query = query.order_by_asc(user::Column::CreatedAt),
        _ => query = query.order_by_asc(user::Column::Id),
    }

    let paginator = query.paginate(db, per_page.into());
    let total = paginator.num_items().await.unwrap_or(0);
    let users = paginator.fetch_page((page - 1) as u64).await.unwrap_or_default();
    let result = users.into_iter().map(RoleResponse::from).collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            PaginatedRoleResponse {
                users: result,
                page,
                per_page,
                total,
            },
            "Personnel retrieved successfully",
        )),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
pub struct EligibleUserQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort: Option<String>,
    pub query: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MinimalUserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub admin: bool,
}

impl From<UserModel> for MinimalUserResponse {
    fn from(user: UserModel) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            admin: user.admin,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EligibleUserListResponse {
    pub users: Vec<MinimalUserResponse>,
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
}

/// GET /api/modules/{module_id}/eligible-users
///
/// Retrieves a paginated list of users who are not assigned to **any role** in the given module.
///
/// This allows administrators to see which users are eligible to be assigned as lecturers, tutors, or students.
///
/// # Arguments
///
/// - `module_id`: Module ID (path param)
/// - `query`, `email`, `username`: optional search filters
/// - `page`: pagination (default = 1)
/// - `per_page`: page size (default = 20, max = 100)
/// - `sort`: sorting field (prefix with `-` for descending). Options: `email`, `username`, `created_at`.
///
/// # Returns
///
/// - `200 OK`: Eligible users and pagination metadata
/// - `500 Internal Server Error`: On DB errors
///
/// # Example Response
/// ```json
/// {
///   "success": true,
///   "data": {
///     "users": [
///       { "id": 1, "username": "u123", "email": "user@example.com", "admin": false }
///     ],
///     "page": 1,
///     "per_page": 20,
///     "total": 45
///   },
///   "message": "Eligible users fetched"
/// }
/// ```
pub async fn get_eligible_users_for_module(
    State(app_state): State<AppState>,
    Path(module_id): Path<i64>,
    Query(params): Query<EligibleUserQuery>,
) -> Response {
    let db = app_state.db();

    let assigned_ids: Vec<i64> = user_module_role::Entity::find()
        .select_only()
        .column(user_module_role::Column::UserId)
        .filter(user_module_role::Column::ModuleId.eq(module_id))
        .into_tuple::<i64>()
        .all(db)
        .await
        .unwrap_or_default();

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    let mut condition = Condition::all();
    if !assigned_ids.is_empty() {
        condition = condition.add(user::Column::Id.is_not_in(assigned_ids));
    }

    if let Some(ref q) = params.query {
        let pattern = format!("%{}%", q.to_lowercase());
        condition = condition.add(
            Condition::any()
                .add(user::Column::Email.contains(&pattern))
                .add(user::Column::Username.contains(&pattern)),
        );
    } else {
        if let Some(ref email) = params.email {
            condition = condition.add(user::Column::Email.contains(email));
        }
        if let Some(ref username) = params.username {
            condition = condition.add(user::Column::Username.contains(username));
        }
    }

    let mut query = user::Entity::find().filter(condition);
    if let Some(sort) = &params.sort {
        let (field, dir) = if sort.starts_with('-') {
            (&sort[1..], Order::Desc)
        } else {
            (sort.as_str(), Order::Asc)
        };

        match field {
            "email" => query = query.order_by(user::Column::Email, dir),
            "username" => query = query.order_by(user::Column::Username, dir),
            "created_at" => query = query.order_by(user::Column::CreatedAt, dir),
            _ => {}
        }
    } else {
        query = query.order_by(user::Column::Id, Order::Asc);
    }

    let paginator = query.paginate(db, per_page.into());
    let total = paginator.num_items().await.unwrap_or(0);
    let raw_users = paginator.fetch_page((page - 1).into()).await.unwrap_or_default();
    let users = raw_users.into_iter().map(MinimalUserResponse::from).collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            EligibleUserListResponse {
                users,
                page,
                per_page,
                total,
            },
            "Eligible users fetched",
        )),
    )
        .into_response()
}