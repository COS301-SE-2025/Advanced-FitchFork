use std::path::PathBuf;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum::http::{header, HeaderMap, HeaderValue};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tokio::fs::File as FsFile;
use serde::Serialize;
use tokio::io::AsyncReadExt;
use crate::{
    auth::claims::AuthUser,
    response::ApiResponse,
};

use db::{
    connect,
    models::{user, module, user_module_role},
};

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub id: i64,
    pub email: String,
    pub student_number: String,
    pub admin: bool,
    pub created_at: String,
    pub updated_at: String,
    pub modules: Vec<UserModuleRoleResponse>,
}

#[derive(Debug, Serialize)]
pub struct UserModuleRoleResponse {
    pub module_id: i64,
    pub module_code: String,
    pub module_year: i32,
    pub module_description: Option<String>,
    pub module_credits: i32,
    pub module_created_at: String,
    pub module_updated_at: String,
    pub role: String,
}

/// GET /api/auth/me
///
/// Returns the authenticated user's profile and module roles using raw SeaORM.
pub async fn get_me(AuthUser(claims): AuthUser) -> impl IntoResponse {
    let db = connect().await;
    let user_id = claims.sub;

    // Find the user directly
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

    // Join user_module_roles with modules
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

    let modules: Vec<UserModuleRoleResponse> = roles
        .into_iter()
        .filter_map(|(role, maybe_module)| {
            maybe_module.map(|m| UserModuleRoleResponse {
                module_id: m.id,
                module_code: m.code,
                module_year: m.year,
                module_description: m.description,
                module_credits: m.credits,
                module_created_at: m.created_at.to_rfc3339(),
                module_updated_at: m.updated_at.to_rfc3339(),
                role: role.role.to_string(),
            })
        })
        .collect();

    let response_data = MeResponse {
        id: user.id,
        email: user.email,
        student_number: user.student_number,
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

pub async fn get_own_avatar(AuthUser(claims): AuthUser) -> impl IntoResponse {
    let db = connect().await;

    let user = match user::Entity::find_by_id(claims.sub).one(&db).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("User not found")),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error")),
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