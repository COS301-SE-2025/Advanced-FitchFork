use std::path::PathBuf;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum::extract::{Query, Path};
use axum::http::{header, HeaderMap, HeaderValue};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tokio::fs::File as FsFile;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
use crate::{
    auth::claims::AuthUser,
    response::ApiResponse,
};

use db::{
    connect,
    models::{user, module, user_module_role},
};
use db::models::user_module_role::Role;
use crate::routes::common::UserModule;

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub admin: bool,
    pub created_at: String,
    pub updated_at: String,
    pub modules: Vec<UserModule>,
}

#[derive(Deserialize)]
pub struct HasRoleQuery {
    pub module_id: i64,
    pub role: String,
}

#[derive(serde::Serialize)]
pub struct HasRoleResponse {
    pub has_role: bool,
}

/// GET /api/auth/me
///
/// Returns the authenticated user's profile along with their module roles.
/// Requires a valid bearer token in the `Authorization` header.
///
/// ### Response: 200 OK
/// ```json
/// {
///   "success": true,
///   "message": "User data retrieved successfully",
///   "data": {
///     "id": 42,
///     "email": "lecturer@example.edu",
///     "username": null,
///     "admin": true,
///     "created_at": "2024-11-10T12:34:56Z",
///     "updated_at": "2025-06-18T10:00:00Z",
///     "modules": [
///       {
///         "module_id": 101,
///         "module_code": "CS101",
///         "module_year": 2025,
///         "module_description": "Intro to Computer Science",
///         "module_credits": 15,
///         "module_created_at": "2023-11-01T08:00:00Z",
///         "module_updated_at": "2025-02-20T14:22:00Z",
///         "role": "Lecturer"
///       },
///       {
///         "module_id": 202,
///         "module_code": "CS202",
///         "module_year": 2025,
///         "module_description": "Data Structures",
///         "module_credits": 20,
///         "module_created_at": "2023-11-05T09:00:00Z",
///         "module_updated_at": "2025-03-10T13:45:00Z",
///         "role": "Admin"
///       }
///     ]
///   }
/// }
/// ```
///
/// ### Error Responses
/// - `403 Forbidden` – Missing or invalid token
/// - `404 Not Found` – User not found
/// - `500 Internal Server Error` – Database failure
pub async fn get_me(AuthUser(claims): AuthUser) -> impl IntoResponse {
    let db: sea_orm::DatabaseConnection = connect().await;
    let user_id = claims.sub;

    let user = match user::Entity::find()
        .filter(user::Column::Id.eq(user_id))
        .one(&db)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<MeResponse>::error("User not found")),
            );
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<MeResponse>::error("Database error")),
            );
        }
    };

    let roles = match user_module_role::Entity::find()
        .filter(user_module_role::Column::UserId.eq(user.id))
        .find_also_related(module::Entity)
        .all(&db)
        .await
    {
        Ok(results) => results,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<MeResponse>::error("Failed to load module roles")),
            );
        }
    };

    let modules: Vec<UserModule> = roles
        .into_iter()
        .filter_map(|(role, maybe_module)| {
            maybe_module.map(|m| UserModule {
                id: m.id,
                code: m.code,
                year: m.year,
                description: m.description.unwrap_or_default(),
                credits: m.credits,
                created_at: m.created_at.to_rfc3339(),
                updated_at: m.updated_at.to_rfc3339(),
                role: role.role.to_string(),
            })
        })
        .collect();

    let response_data = MeResponse {
        id: user.id,
        email: user.email,
        username: user.username,
        admin: user.admin,
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.updated_at.to_rfc3339(),
        modules,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(response_data, "User data retrieved successfully")),
    )
}

/// GET /api/auth/avatar/:user_id
///
/// Returns the avatar image for a specific user ID.
///
/// ### Authorization
/// This endpoint is **public** and does **not** require authentication.
///
/// ### Response: 200 OK
/// - Returns raw binary image data of the avatar
/// - The `Content-Type` header is automatically inferred based on the file extension (e.g., `image/png`, `image/jpeg`)
///
/// ### Example Request
/// ```http
/// GET /api/auth/avatar/42
/// ```
///
/// ### Example Response Headers
/// ```http
/// Content-Type: image/png
/// ```
///
/// ### Error Responses
/// #### 404 Not Found
/// - User does not exist
/// - No avatar is set
/// - Avatar file is missing
/// ```json
/// {
///   "success": false,
///   "message": "No avatar set",
///   "data": null
/// }
/// ```
///
/// #### 500 Internal Server Error
/// - Database error
/// - File could not be opened or read
/// ```json
/// {
///   "success": false,
///   "message": "Failed to read avatar",
///   "data": null
/// }
/// ```
pub async fn get_avatar(Path(user_id): Path<i64>) -> impl IntoResponse {
    let db = connect().await;
    let user = match user::Entity::find_by_id(user_id).one(&db).await {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("User not found")),
            )
                .into_response();
        }
    };

    let Some(path) = user.profile_picture_path else {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("No avatar set")),
        )
            .into_response();
    };

    let root = std::env::var("USER_PROFILE_STORAGE_ROOT")
        .unwrap_or_else(|_| "data/user_profile_pictures".to_string());
    let fs_path = PathBuf::from(root).join(path);

    if tokio::fs::metadata(&fs_path).await.is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Avatar file missing")),
        )
            .into_response();
    }

    let mut file = match FsFile::open(&fs_path).await {
        Ok(f) => f,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Could not open avatar")),
            )
                .into_response();
        }
    };

    let mut buffer = Vec::new();
    if let Err(_) = file.read_to_end(&mut buffer).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to read avatar")),
        )
            .into_response();
    }

    let mime = mime_guess::from_path(&fs_path)
        .first_or_octet_stream()
        .to_string();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&mime).unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );

    (StatusCode::OK, headers, buffer).into_response()
}

/// GET /api/auth/has-role
///
/// Checks if the authenticated user has a specific role in a module.
///
/// ### Authorization
/// This endpoint requires a valid bearer token in the `Authorization` header.
///
/// ### Request Parameters
/// - `module_id`: The ID of the module to check role for
/// - `role`: The role to check for (case-insensitive: "lecturer", "tutor", "student")
///
/// ### Response: 200 OK
/// ```json
/// {
///   "success": true,
///   "message": "Role check completed",
///   "data": {
///     "has_role": true
/// }
/// ```
///
/// ### Error Responses
/// - `400 Bad Request` – Invalid role specified
/// - `403 Forbidden` – Missing or invalid token
/// - `500 Internal Server Error` – Database failure
pub async fn has_role_in_module(AuthUser(claims): AuthUser, Query(params): Query<HasRoleQuery>, ) -> impl IntoResponse {
    let db = connect().await;

    let role = match params.role.to_lowercase().as_str() {
        "lecturer" => Role::Lecturer,
        "assistant_lecturer" => Role::AssistantLecturer,
        "tutor" => Role::Tutor,
        "student" => Role::Student,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<HasRoleResponse>::error("Invalid role specified")),
            )
        }
    };

    let exists = match user_module_role::Entity::find()
        .filter(user_module_role::Column::UserId.eq(claims.sub))
        .filter(user_module_role::Column::ModuleId.eq(params.module_id))
        .filter(user_module_role::Column::Role.eq(role))
        .one(&db)
        .await
    {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<HasRoleResponse>::error("Database error")),
            )
        }
    };

    let response = HasRoleResponse { has_role: exists };

    (
        StatusCode::OK,
        Json(ApiResponse::success(response, "Role check completed")),
    )
}



#[derive(Debug, Deserialize)]
pub struct ModuleRoleQuery {
    pub module_id: i32,
}

#[derive(Debug, Serialize)]
pub struct ModuleRoleResponse {
    pub role: Option<String>,
}
/// GET /api/auth/module-role
///
/// Returns the role of the authenticated user in the given module, if any.
///
/// ### Authorization
/// Requires a valid bearer token.
///
/// ### Query Parameters
/// - `module_id`: The module ID to check
///
/// ### Response
/// ```json
/// {
///   "success": true,
///   "message": "Role fetched successfully",
///   "data": {
///     "role": "lecturer" // or "tutor", "student", null
///   }
/// }
/// ```
/// #[derive(Debug, Deserialize)]
pub async fn get_module_role(
    AuthUser(claims): AuthUser,
    Query(params): Query<ModuleRoleQuery>,
) -> impl IntoResponse {
    let db = connect().await;

    let role = match user_module_role::Entity::find()
        .filter(user_module_role::Column::UserId.eq(claims.sub))
        .filter(user_module_role::Column::ModuleId.eq(params.module_id))
        .one(&db)
        .await
    {
        Ok(Some(model)) => Some(model.role.to_string().to_lowercase()),
        Ok(None) => None,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<ModuleRoleResponse>::error("Database error")),
            );
        }
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            ModuleRoleResponse { role },
            "Role fetched successfully",
        )),
    )
}