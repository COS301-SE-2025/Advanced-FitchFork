//! # User Creation Routes
//!
//! - `POST /api/users`: Create a single non-admin user
//! - `POST /api/users/bulk`: Create multiple non-admin users
//!
//! All routes require admin privileges.

use crate::response::ApiResponse;
use crate::routes::users::common::{BulkCreateUsersRequest, CreateUserRequest, UserResponse};
use axum::{Json, http::StatusCode, response::IntoResponse};
use services::service::Service;
use services::user::{CreateUser, UserService};
use validator::Validate;

/// POST /api/users
///
/// Creates a single **non-admin** user. Admin-only access.
///
/// ### Request Body
/// ```json
/// {
///   "username": "u12345678",
///   "email": "test@example.com",
///   "password": "securepassword"
/// }
/// ```
///
/// ### Response: 201 Created
/// - JSON body with full user object (excluding password)
///
/// ### Errors:
/// - 400 Bad Request — Validation failure
/// - 409 Conflict — Duplicate username/email
pub async fn create_user(Json(req): Json<CreateUserRequest>) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(format!("Validation failed: {e}"))),
        )
            .into_response();
    }

    match UserService::create(CreateUser {
        id: None,
        username: req.username,
        email: req.email,
        password: req.password,
        admin: false,
    })
    .await
    {
        Ok(user) => (
            StatusCode::CREATED,
            Json(ApiResponse::<UserResponse>::success(
                user.into(),
                "User created successfully",
            )),
        )
            .into_response(),
        Err(e) => {
            let msg = if e.to_string().contains("UNIQUE constraint failed") {
                "A user with this username or email already exists".to_string()
            } else {
                format!("Database error: {e}")
            };
            (StatusCode::CONFLICT, Json(ApiResponse::<()>::error(msg))).into_response()
        }
    }
}

/// POST /api/users/bulk
///
/// Creates multiple **non-admin** users. Admin-only access.
///
/// ### Request Body
/// ```json
/// {
///   "users": [
///     { "username": "u001", "email": "a@x.com", "password": "pw1" },
///     { "username": "u002", "email": "b@x.com", "password": "pw2" }
///   ]
/// }
/// ```
///
/// ### Response: 201 Created
/// - JSON array of created user objects
///
/// ### Errors:
/// - 400 Bad Request — If validation fails
/// - 409 Conflict — If one user fails to insert (first error returned)
pub async fn bulk_create_users(Json(req): Json<BulkCreateUsersRequest>) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(format!("Validation failed: {e}"))),
        )
            .into_response();
    }

    let mut results = Vec::new();

    for user_req in req.users {
        match UserService::create(CreateUser {
            id: None,
            username: user_req.username.clone(),
            email: user_req.email,
            password: user_req.password,
            admin: false,
        })
        .await
        {
            Ok(u) => results.push(UserResponse::from(u)),
            Err(e) => {
                let msg = if e.to_string().contains("UNIQUE constraint failed") {
                    format!(
                        "A user with this username or email already exists: {}",
                        user_req.username
                    )
                } else {
                    format!("Database error while creating {}: {}", user_req.username, e)
                };

                return (StatusCode::CONFLICT, Json(ApiResponse::<()>::error(msg))).into_response();
            }
        }
    }

    (
        StatusCode::CREATED,
        Json(ApiResponse::<Vec<UserResponse>>::success(
            results,
            "Users created successfully",
        )),
    )
        .into_response()
}
