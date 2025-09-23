use crate::response::ApiResponse;
use crate::routes::common::UserModule;
use axum::body::Body;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{Response, StatusCode},
    response::IntoResponse,
};
use db::models::user::{Column as UserColumn, Entity as UserEntity, Model as UserModel};
use mime_guess::from_path;
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use util::{paths::user_profile_path, state::AppState};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct ListUsersQuery {
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<u64>,
    pub sort: Option<String>,
    pub query: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub admin: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UserListItem {
    pub id: String,
    pub email: String,
    pub username: String,
    pub admin: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct UsersListResponse {
    pub users: Vec<UserListItem>,
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
}

impl From<UserModel> for UserListItem {
    fn from(user: UserModel) -> Self {
        Self {
            id: user.id.to_string(),
            email: user.email,
            username: user.username,
            admin: user.admin,
            created_at: user.created_at.to_string(),
            updated_at: user.updated_at.to_string(),
        }
    }
}

/// GET /api/users
///
/// Retrieve a paginated list of users with optional filtering and sorting.
/// Requires admin privileges.
///
/// ### Query Parameters
/// - `page` (optional): Page number (default: 1, min: 1)
/// - `per_page` (optional): Items per page (default: 20, min: 1, max: 100)
/// - `query` (optional): Case-insensitive partial match against email OR username
/// - `email` (optional): Case-insensitive partial match on email
/// - `username` (optional): Case-insensitive partial match on student number
/// - `admin` (optional): Filter by admin status (true/false)
/// - `sort` (optional): Comma-separated sort fields. Use `-` prefix for descending
///
/// ### Examples
/// ```http
/// GET /api/users?page=2&per_page=10
/// GET /api/users?query=u1234
/// GET /api/users?email=@example.com
/// GET /api/users?username=u1234
/// GET /api/users?admin=true
/// GET /api/users?sort=email,-created_at
/// GET /api/users?page=1&per_page=10&admin=false&query=jacques&sort=-email
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
///         "id": "uuid",
///         "email": "user@example.com",
///         "username": "u12345678",
///         "admin": false,
///         "created_at": "2025-05-23T18:00:00Z",
///         "updated_at": "2025-05-23T18:00:00Z"
///       }
///     ],
///     "page": 1,
///     "per_page": 10,
///     "total": 135
///   },
///   "message": "Users retrieved successfully"
/// }
/// ```
///
/// - `400 Bad Request` - Invalid query parameters
/// - `401 Unauthorized` - Missing or invalid JWT
/// - `403 Forbidden` - Authenticated but not admin user
/// - `500 Internal Server Error` - Database error
pub async fn list_users(
    State(app_state): State<AppState>,
    Query(query): Query<ListUsersQuery>,
) -> impl IntoResponse {
    let db = app_state.db();

    if let Err(e) = query.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<UsersListResponse>::error(
                common::format_validation_errors(&e),
            )),
        );
    }

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    let mut condition = Condition::all();

    if let Some(q) = &query.query {
        let pattern = format!("%{}%", q.to_lowercase());
        condition = condition.add(
            Condition::any()
                .add(UserColumn::Email.contains(&pattern))
                .add(UserColumn::Username.contains(&pattern)),
        );
    }

    if let Some(email) = &query.email {
        condition = condition.add(UserColumn::Email.contains(&format!("%{}%", email)));
    }

    if let Some(sn) = &query.username {
        condition = condition.add(UserColumn::Username.contains(&format!("%{}%", sn)));
    }

    if let Some(admin) = query.admin {
        condition = condition.add(UserColumn::Admin.eq(admin));
    }

    let mut query_builder = UserEntity::find().filter(condition);

    if let Some(sort_param) = &query.sort {
        for sort_field in sort_param.split(',') {
            let (field, desc) = if sort_field.starts_with('-') {
                (&sort_field[1..], true)
            } else {
                (sort_field, false)
            };

            match field {
                "email" => {
                    query_builder = if desc {
                        query_builder.order_by_desc(UserColumn::Email)
                    } else {
                        query_builder.order_by_asc(UserColumn::Email)
                    };
                }
                "username" => {
                    query_builder = if desc {
                        query_builder.order_by_desc(UserColumn::Username)
                    } else {
                        query_builder.order_by_asc(UserColumn::Username)
                    };
                }
                "created_at" => {
                    query_builder = if desc {
                        query_builder.order_by_desc(UserColumn::CreatedAt)
                    } else {
                        query_builder.order_by_asc(UserColumn::CreatedAt)
                    };
                }
                "admin" => {
                    query_builder = if desc {
                        query_builder.order_by_desc(UserColumn::Admin)
                    } else {
                        query_builder.order_by_asc(UserColumn::Admin)
                    };
                }
                _ => {}
            }
        }
    } else {
        query_builder = query_builder.order_by_asc(UserColumn::Id);
    }

    let paginator = query_builder.paginate(db, per_page);
    let total = paginator.num_items().await.unwrap_or(0);
    let users = paginator.fetch_page(page - 1).await.unwrap_or_default();
    let users = users.into_iter().map(UserListItem::from).collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            UsersListResponse {
                users,
                page,
                per_page,
                total,
            },
            "Users retrieved successfully",
        )),
    )
}

/// GET /api/users/{user_id}
///
/// Fetch a single user by ID. Requires admin privileges.
///
/// ### Path Parameters
/// - `id`: The user ID (integer)
///
/// ### Responses
/// - `200 OK`: User found
/// - `400 Bad Request`: Invalid ID format
/// - `404 Not Found`: User does not exist
/// - `500 Internal Server Error`: DB error
pub async fn get_user(
    State(app_state): State<AppState>,
    Path(user_id): Path<i64>,
) -> impl IntoResponse {
    let db = app_state.db();

    match UserEntity::find_by_id(user_id).one(db).await {
        Ok(Some(user)) => {
            let user_item = UserListItem::from(user);
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    user_item,
                    "User retrieved successfully",
                )),
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<UserListItem>::error("User not found")),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<UserListItem>::error(format!(
                "Database error: {}",
                err
            ))),
        ),
    }
}

/// GET /api/users/{user_id}/modules
///
/// Retrieve all modules that a specific user is involved in, including their role in each module.
/// Requires admin privileges.
///
/// ### Path Parameters
/// - `id`: The ID of the user to fetch modules for
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "data": [
///     {
///       "id": 1,
///       "code": "COS301",
///       "year": 2025,
///       "description": "Advanced Software Engineering",
///       "credits": 16,
///       "role": "Lecturer",
///       "created_at": "2025-05-01T08:00:00Z",
///       "updated_at": "2025-05-01T08:00:00Z"
///     }
///   ],
///   "message": "Modules for user retrieved successfully"
/// }
/// ```
///
/// - `400 Bad Request` (invalid ID format)
/// ```json
/// {
///   "success": false,
///   "message": "Invalid user ID format"
/// }
/// ```
///
/// - `403 Forbidden` - Not an admin user
/// - `404 Not Found` - User not found
/// ```json
/// {
///   "success": false,
///   "message": "User not found"
/// }
/// ```
///
/// - `500 Internal Server Error` - Database error
/// ```json
/// {
///   "success": false,
///   "message": "Database error: detailed error here"
/// }
/// ```
pub async fn get_user_modules(
    State(app_state): State<AppState>,
    Path(user_id): Path<i64>,
) -> impl IntoResponse {
    let db = app_state.db();

    let roles = match UserModel::get_module_roles(db, user_id).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<UserModule>>::error(format!(
                    "Database error: {}",
                    e
                ))),
            );
        }
    };

    let modules = roles
        .into_iter()
        .map(|r| UserModule {
            id: r.module_id,
            code: r.module_code,
            year: r.module_year,
            description: r.module_description.unwrap_or_default(),
            credits: r.module_credits,
            role: r.role,
            created_at: r.module_created_at,
            updated_at: r.module_updated_at,
        })
        .collect::<Vec<_>>();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            modules,
            "Modules for user retrieved successfully",
        )),
    )
}

/// GET /api/users/{user_id}/avatar
///
/// Returns the avatar image file for a user if it exists.
pub async fn get_avatar(
    State(_state): State<AppState>,
    Path(user_id): Path<i64>,
) -> impl IntoResponse {
    // Try common extensions under the user's profile dir: .../user_{id}/profile/avatar.{ext}
    for ext in ["jpg", "png", "gif"] {
        let try_path = user_profile_path(user_id, &format!("avatar.{ext}"));
        if tokio::fs::metadata(&try_path).await.is_ok() {
            let file = match File::open(&try_path).await {
                Ok(f) => f,
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

            let mime = from_path(&try_path).first_or_octet_stream();
            let stream = ReaderStream::new(file);
            let body = Body::from_stream(stream);

            return Response::builder()
                .header("Content-Type", mime.as_ref())
                .body(body)
                .unwrap();
        }
    }

    StatusCode::NOT_FOUND.into_response()
}
