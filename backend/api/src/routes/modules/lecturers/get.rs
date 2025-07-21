use axum::{extract::{Path, Query, State}, http::StatusCode, response::{Response, IntoResponse}, Json};
use sea_orm::{EntityTrait, QueryFilter, Condition, ColumnTrait, JoinType, PaginatorTrait, DatabaseConnection, QuerySelect, QueryOrder};
use crate::response::ApiResponse;
use db::models::{user, user_module_role::{self, Column as RoleCol, Role}};
use crate::routes::modules::common::{RoleResponse, RoleQuery, PaginatedRoleResponse};

/// GET /api/modules/{module_id}/lecturers
///
/// Retrieve a paginated, filtered, and sorted list of users assigned as lecturers to the specified module.
///
/// ### Access Control
/// This endpoint is accessible to:
/// - Admin users (`claims.admin == true`)
/// - Users assigned to the module (as Lecturer, Tutor, or Student)
///
/// ### Path Parameters
/// - `module_id` (integer): The ID of the module whose lecturers are being queried.
///
/// ### Query Parameters (All Optional)
/// - `page` (integer): Page number, default is `1`, must be ≥ 1.
/// - `per_page` (integer): Number of results per page, default is `20`, max is `100`.
/// - `query` (string): Fuzzy search term for `email` or `username` (case-insensitive).
/// - `email` (string): Filter by email (case-insensitive, ignored if `query` is present).
/// - `username` (string): Filter by student number (ignored if `query` is present).
/// - `sort` (string): Sort by field. Prefix with `-` for descending order. Allowed fields:
///   - `email`
///   - `username`
///   - `created_at`
///
/// ### Authentication
/// Requires a valid JWT with appropriate permissions. Returns `403` if the user is not an admin and not assigned to the module.
///
/// ### Example Requests
/// ```http
/// GET /api/modules/42/lecturers?page=2&per_page=10
/// GET /api/modules/42/lecturers?query=example
/// GET /api/modules/42/lecturers?email=@up.ac.za
/// GET /api/modules/42/lecturers?sort=-created_at
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
///         "id": 1,
///         "username": "u12345678",
///         "email": "lecturer@example.com",
///         "admin": false,
///         "created_at": "2025-05-23T18:00:00Z",
///         "updated_at": "2025-05-23T18:00:00Z"
///       }
///     ],
///     "page": 1,
///     "per_page": 20,
///     "total": 57
///   },
///   "message": "Lecturers retrieved successfully"
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
pub async fn get_lecturers(
    State(db): State<DatabaseConnection>,
    Path(module_id): Path<i32>,
    Query(params): Query<RoleQuery>,
) -> Response {
    let module_exists = user_module_role::Entity::find()
        .filter(RoleCol::ModuleId.eq(module_id))
        .one(&db)
        .await;

    if let Ok(None) | Err(_) = module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        )
            .into_response();
    }

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);

    let mut condition = Condition::all()
        .add(RoleCol::ModuleId.eq(module_id))
        .add(RoleCol::Role.eq(Role::Lecturer));

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
        .filter(condition.clone());

    match params.sort.as_deref() {
        Some("-email") => query = query.order_by_desc(user::Column::Email),
        Some("email") => query = query.order_by_asc(user::Column::Email),
        Some("-username") => query = query.order_by_desc(user::Column::Username),
        Some("username") => query = query.order_by_asc(user::Column::Username),
        Some("-created_at") => query = query.order_by_desc(user::Column::CreatedAt),
        Some("created_at") => query = query.order_by_asc(user::Column::CreatedAt),
        _ => query = query.order_by_asc(user::Column::Id),
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
            "Lecturers retrieved successfully",
        )),
    )
        .into_response()
}