//! POST handlers for `/api/test`.

use axum::{http::StatusCode, response::IntoResponse, Json};
use validator::Validate;
use crate::response::ApiResponse;
use super::common::{TestUserResponse, UpsertUserRequest};
use services::service::Service;
use services::user::{UserService, CreateUser, UpdateUser};
use util::filters::FilterParam;

/// POST `/api/test/users`
///
/// Create-or-update (**idempotent**) a user.  
/// - If the `username` exists, updates `email`, `password`, and `admin`.
/// - If it does not exist, creates a new user.
/// - This endpoint is for **test environments only**.
///
/// # Request body
/// ```json
/// {
///   "username": "student42",
///   "email": "student42@example.com",
///   "password": "secret",
///   "admin": false
/// }
/// ```
///
/// ## Field notes
/// - `admin` is optional and defaults to `false`.
/// - `password` is stored as a hashed value.
///
/// # Response examples
///
/// ## 201 Created (new user)
/// ```json
/// {
///   "success": true,
///   "data": {
///     "id": 1,
///     "username": "student42",
///     "email": "student42@example.com",
///     "admin": false
///   },
///   "message": "User created"
/// }
/// ```
///
/// ## 200 OK (updated existing user)
/// ```json
/// {
///   "success": true,
///   "data": {
///     "id": 1,
///     "username": "student42",
///     "email": "new@example.com",
///     "admin": true
///   },
///   "message": "User updated"
/// }
/// ```
///
/// ## 400 Bad Request (validation error)
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Validation failed: email: must be a valid email"
/// }
/// ```
///
/// ## 409 Conflict (username/email already exists)
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "A user with this username or email already exists"
/// }
/// ```
///
/// ## 500 Internal Server Error
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Database error: <details>"
/// }
/// ```
pub async fn upsert_user(
    Json(req): Json<UpsertUserRequest>,
) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(format!("Validation failed: {e}"))),
        )
            .into_response();
    }

    let admin = req.admin.unwrap_or(false);

    match UserService::find_one(
        &vec![
            FilterParam::eq("username", req.username),
        ],
        &vec![],
        None,
    ).await {
        Ok(Some(existing)) => {
            // Update existing user
            match UserService::update(
                UpdateUser{
                    id: existing.id,
                    username: None,
                    email: Some(req.email.clone()),
                    password: Some(req.password.clone()),
                    admin: Some(admin),
                    profile_picture_path: None,
                }
            ).await {
                Ok(updated) => (
                    StatusCode::OK,
                    Json(ApiResponse::success(
                        TestUserResponse::from(updated),
                        String::from("User updated"),
                    )),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(format!("Database error: {e}"))),
                )
                    .into_response(),
            }
        }
        Ok(None) => {
            // Create new user
            match UserService::create(
                CreateUser{
                    id: None,
                    username: req.username.clone(),
                    email: req.email.clone(),
                    password: req.password.clone(),
                    admin,
                }
            ).await {
                Ok(user) => (
                    StatusCode::CREATED,
                    Json(ApiResponse::success(
                        TestUserResponse::from(user),
                        String::from("User created"),
                    )),
                )
                    .into_response(),
                Err(e) => {
                    let msg = if e.to_string().contains("UNIQUE constraint failed")
                        || e.to_string().to_lowercase().contains("unique")
                    {
                        String::from("A user with this username or email already exists")
                    } else {
                        format!("Database error: {e}")
                    };
                    (
                        StatusCode::CONFLICT,
                        Json(ApiResponse::<()>::error(msg)),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {e}"))),
        )
            .into_response(),
    }
}
