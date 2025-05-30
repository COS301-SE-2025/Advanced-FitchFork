use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use serde::Serialize;

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
