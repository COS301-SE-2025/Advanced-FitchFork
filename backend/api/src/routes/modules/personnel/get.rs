use crate::{
    auth::AuthUser,
    response::ApiResponse,
    routes::modules::common::{PaginatedRoleResponse, RoleQuery, RoleResponse},
};
use axum::{
    Json,
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use services::service::Service;
use services::user::{User, UserService};
use services::user_module_role::UserModuleRoleService;
use util::filters::{FilterParam, QueryParam};

#[derive(Debug, serde::Deserialize)]
pub struct RoleParam {
    pub role: String,
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
    Path(module_id): Path<i64>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Query(role_param): Query<RoleParam>,
    Query(params): Query<RoleQuery>,
) -> impl IntoResponse {
    let user_id = claims.sub;
    let requested_role = &role_param.role;

    if !["lecturer", "tutor", "assistant_lecturer", "student"].contains(&requested_role.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<PaginatedRoleResponse>::error(
                "Invalid role specified",
            )),
        );
    }

    if !claims.admin {
        if requested_role == "lecturer" {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::<PaginatedRoleResponse>::error(
                    "Only admins can view lecturers",
                )),
            );
        }

        match UserModuleRoleService::find_one(
            &vec![
                FilterParam::eq("user_id", user_id),
                FilterParam::eq("module_id", module_id),
            ],
            &vec![],
            None,
        )
        .await
        {
            Ok(is_member) => {
                if is_member.is_none() {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(ApiResponse::<PaginatedRoleResponse>::error(
                            "You do not have permission to view this module's users",
                        )),
                    );
                }
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<PaginatedRoleResponse>::error(format!(
                        "Database error: {}",
                        e
                    ))),
                );
            }
        }
    }

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let sort = params.sort.clone();

    let mut filters = vec![
        FilterParam::eq("module_id", module_id),
        FilterParam::eq("role", requested_role.clone()),
    ];

    let mut queries = Vec::new();

    if let Some(query_text) = params.query {
        queries.push(QueryParam::new(
            vec!["email".to_string(), "username".to_string()],
            query_text,
        ));
    } else {
        if let Some(email) = params.email {
            filters.push(FilterParam::like("email", email));
        }
        if let Some(username) = params.username {
            filters.push(FilterParam::like("username", username));
        }
    }

    let (users, total) = match UserService::filter(&filters, &queries, page, per_page, sort).await {
        Ok(result) => result,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<PaginatedRoleResponse>::error(format!(
                    "Database error: {}",
                    e
                ))),
            );
        }
    };

    let user_responses: Vec<RoleResponse> = users.into_iter().map(RoleResponse::from).collect();

    let response = PaginatedRoleResponse {
        users: user_responses,
        page,
        per_page,
        total,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            response,
            "Personnel retrieved successfully",
        )),
    )
}

#[derive(Debug, Deserialize)]
pub struct EligibleUserQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
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

impl From<User> for MinimalUserResponse {
    fn from(user: User) -> Self {
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
    pub page: u64,
    pub per_page: u64,
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
    Path(module_id): Path<i64>,
    Query(params): Query<EligibleUserQuery>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let sort = params.sort.clone();

    let assigned_ids = match UserModuleRoleService::find_all(
        &vec![FilterParam::eq("module_id", module_id)],
        &vec![],
        None,
    )
    .await
    {
        Ok(models) => models.into_iter().map(|m| m.user_id).collect::<Vec<i64>>(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<EligibleUserListResponse>::error(format!(
                    "Database error: {}",
                    e
                ))),
            );
        }
    };

    let mut filters = Vec::new();
    let mut queries = Vec::new();

    if !assigned_ids.is_empty() {
        filters.push(FilterParam::ne("id", assigned_ids));
    }

    if let Some(query_text) = params.query {
        queries.push(QueryParam::new(
            vec!["email".to_string(), "username".to_string()],
            query_text,
        ));
    } else {
        if let Some(email) = params.email {
            filters.push(FilterParam::like("email", email));
        }
        if let Some(username) = params.username {
            filters.push(FilterParam::like("username", username));
        }
    }

    let (users, total) = match UserService::filter(&filters, &queries, page, per_page, sort).await {
        Ok(result) => result,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<EligibleUserListResponse>::error(format!(
                    "Database error: {}",
                    e
                ))),
            );
        }
    };

    let user_responses: Vec<MinimalUserResponse> =
        users.into_iter().map(MinimalUserResponse::from).collect();

    let response = EligibleUserListResponse {
        users: user_responses,
        page,
        per_page,
        total,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(response, "Eligible users fetched")),
    )
}
