use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use crate::response::ApiResponse;
use db::models::{
    user,
    user_module_role,
    user_module_role::{Column as RoleCol, Role},
};
use sea_orm::{EntityTrait, QueryFilter, Condition, ColumnTrait, JoinType, QuerySelect, QueryOrder, DatabaseConnection, PaginatorTrait};
use crate::routes::modules::common::{RoleResponse, RoleQuery, PaginatedRoleResponse};

/// GET /api/modules/{module_id}/tutors
///
/// Retrieve a paginated, filtered, and sorted list of users assigned as tutors to the specified module.
///
/// ### Access Control
/// This endpoint is accessible to:
/// - Admin users (`claims.admin == true`)
/// - Users assigned to the module (as Lecturer, Tutor, or Student)
///
/// ### Path Parameters
/// - `module_id` (integer): The ID of the module whose tutors are being queried.
///
/// ### Query Parameters (All Optional)
/// - `page` (integer): Page number. Default is `1`. Must be ≥ 1.
/// - `per_page` (integer): Number of results per page. Default is `20`. Maximum is `100`.
/// - `query` (string): Fuzzy search term for `email` or `username` (case-insensitive).
/// - `email` (string): Filter by email (case-insensitive, ignored if `query` is present).
/// - `username` (string): Filter by student number (ignored if `query` is present).
/// - `sort` (string): Sort by field. Prefix with `-` for descending order. Allowed fields:
///   - `email`
///   - `username`
///   - `created_at`
///
/// ### Authentication
/// Requires a valid JWT. Returns `403 Forbidden` if the user is not an admin and not assigned to the module.
///
/// ### Example Requests
/// ```http
/// GET /api/modules/42/tutors?page=2&per_page=10
/// GET /api/modules/42/tutors?query=up.ac.za
/// GET /api/modules/42/tutors?email=tutor@example.com
/// GET /api/modules/42/tutors?sort=-created_at
/// ```
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": {
///     "users": [
///       {
///         "id": 7,
///         "username": "u22222222",
///         "email": "tutor@example.com",
///         "admin": false,
///         "created_at": "2025-05-23T18:00:00Z",
///         "updated_at": "2025-05-23T18:00:00Z"
///       }
///     ],
///     "page": 1,
///     "per_page": 20,
///     "total": 42
///   },
///   "message": "Tutors retrieved successfully"
/// }
/// ```
///
/// - `403 Forbidden`  
/// ```json
/// {
///   "success": false,
///   "message": "You do not have permission to view this module's users"
/// }
/// ```
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "message": "Module not found"
/// }
/// ```
///
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "message": "An internal server error occurred"
/// }
/// ```
pub async fn get_tutors(
    State(db): State<DatabaseConnection>,
    Path(module_id): Path<i64>,
    Query(params): Query<RoleQuery>,
) -> Response {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    let mut condition = Condition::all()
        .add(RoleCol::ModuleId.eq(module_id))
        .add(RoleCol::Role.eq(Role::Tutor));

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
        .filter(condition.clone());

    if let Some(ref sort) = params.sort {
        let (field, dir) = if sort.starts_with('-') {
            (&sort[1..], sea_orm::Order::Desc)
        } else {
            (sort.as_str(), sea_orm::Order::Asc)
        };

        match field {
            "email" => query = query.order_by(user::Column::Email, dir),
            "username" => query = query.order_by(user::Column::Username, dir),
            "created_at" => query = query.order_by(user::Column::CreatedAt, dir),
            _ => {}
        }
    } else {
        query = query.order_by(user::Column::Id, sea_orm::Order::Asc);
    }

    let paginator = query.paginate(&db, per_page.into());
    let total = paginator.num_items().await.unwrap_or(0);
    let users = paginator.fetch_page((page - 1).into()).await.unwrap_or_default();

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
            "Tutors retrieved successfully",
        )),
    )
        .into_response()
}