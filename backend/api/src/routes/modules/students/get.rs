use axum::{extract::{State, Path, Query}, http::StatusCode, response::{Response, IntoResponse}, Json};
use sea_orm::{EntityTrait, QueryFilter, Condition, ColumnTrait, DatabaseConnection, JoinType, Order, PaginatorTrait, QuerySelect, QueryOrder};
use crate::{
    response::ApiResponse,
};
use db::{
    models::{
        user,
        user_module_role::{Column as RoleCol, Entity as RoleEntity, Role},
    },
};
use crate::routes::modules::common::{RoleResponse, RoleQuery, PaginatedRoleResponse};

/// GET /api/modules/{module_id}/students
///
/// Retrieve a paginated, filtered, and sortable list of users enrolled as students in the specified module.
///
/// ### Access Control
/// This endpoint is accessible to:
/// - Admin users
///
/// ### Path Parameter
/// - `module_id` (integer): The ID of the module to retrieve students for.
///
/// ### Query Parameters
/// - `page` (optional): Page number (default: 1)
/// - `per_page` (optional): Items per page (default: 20, max: 100)
/// - `query` (optional): Case-insensitive partial match against email or student number
/// - `email` (optional): Partial match on email (used only if `query` is not provided)
/// - `username` (optional): Partial match on student number (used only if `query` is not provided)
/// - `sort` (optional): Sort by field. Prefix with `-` for descending. Allowed fields: `email`, `username`, `created_at`
///
/// ### Responses
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "data": {
///     "users": [ { "id": 1, "email": "...", ... } ],
///     "page": 1,
///     "per_page": 20,
///     "total": 87
///   },
///   "message": "Students retrieved successfully"
/// }
/// ```
/// - `403 Forbidden` – if user is not admin or assigned to module
/// - `404 Not Found` – if the module does not exist
pub async fn get_students(
    State(db): State<DatabaseConnection>,
    Path(module_id): Path<i32>,
    Query(params): Query<RoleQuery>,
) -> Response {
    let module_exists = db::models::module::Entity::find_by_id(module_id)
        .one(&db)
        .await
        .unwrap_or(None)
        .is_some();

    if !module_exists {
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
        .add(RoleCol::Role.eq(Role::Student));

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
            user::Entity::belongs_to(RoleEntity)
                .from(user::Column::Id)
                .to(RoleCol::UserId)
                .into(),
        )
        .filter(condition);

    if let Some(ref sort) = params.sort {
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
            "Students retrieved successfully",
        )),
    )
        .into_response()
}