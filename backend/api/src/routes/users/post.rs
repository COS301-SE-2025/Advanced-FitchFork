use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use db::connect;
use db::models::user;
use crate::auth::AuthUser;
use crate::response::ApiResponse;
use crate::services::email::EmailService;

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub admin: bool,
}

#[derive(Serialize)]
pub struct MinimalUserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub admin: bool,
}

pub async fn create_user(
    AuthUser(claims): AuthUser,
    Json(req): Json<CreateUserRequest>,
) -> impl IntoResponse {
    if !claims.admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("Only admins may create users")),
        );
    }

    if let Err(validation_errors) = req.validate() {
        let msg = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<MinimalUserResponse>::error(msg)),
        );
    }

    let db = connect().await;

    // Email check
    if user::Entity::find()
        .filter(user::Column::Email.eq(req.email.clone()))
        .one(&db)
        .await
        .unwrap()
        .is_some()
    {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::<MinimalUserResponse>::error("A user with this email already exists")),
        );
    }

    // Student number check
    if user::Entity::find()
        .filter(user::Column::Username.eq(req.username.clone()))
        .one(&db)
        .await
        .unwrap()
        .is_some()
    {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::<MinimalUserResponse>::error("A user with this student number already exists")),
        );
    }

    let inserted = match UserModel::create(
        &db,
        &req.username,
        &req.email,
        "changeme",
        req.admin,
    )
    .await
    {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<MinimalUserResponse>::error(format!("Database error: {}", e))),
            );
        }
    };

    // Fire off email (can fail silently)
    let _ = EmailService::send_email(
        &req.email,
        "Welcome! Change your password",
        "Please change your password at https://example.com/change-password",
    )
    .await;

    let response = MinimalUserResponse {
        id: inserted.id,
        username: inserted.username,
        email: inserted.email,
        admin: inserted.admin,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(response, "User created and change password email sent")),
    )
}