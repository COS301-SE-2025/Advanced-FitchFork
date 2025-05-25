use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use db::models::user::{User, UserModuleRole};
use db::pool;
use crate::auth::claims::AuthUser;
use crate::response::ApiResponse;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub id: i64,
    pub email: String,
    pub student_number: String,
    pub admin: bool,
    pub created_at: String,
    pub updated_at: String,
    pub modules: Vec<UserModuleRole>,
}

/// GET /api/auth/me
///
/// Returns the authenticated user's profile and module roles.
pub async fn get_me(AuthUser(claims): AuthUser) -> impl IntoResponse {
    let pool = pool::get();
    let user_id = claims.sub;

    let user = match User::get_by_id(Some(pool), user_id).await {
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

    let modules = match User::get_module_roles(Some(pool), user.id).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<MeResponse>::error("Failed to load module roles")),
            );
        }
    };

    let response_data = MeResponse {
        id: user.id,
        email: user.email,
        student_number: user.student_number,
        admin: user.admin,
        created_at: user.created_at,
        updated_at: user.updated_at,
        modules,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(response_data, "User data retrieved successfully")),
    )
}