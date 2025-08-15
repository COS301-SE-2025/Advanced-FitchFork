//! POST handlers for `/api/test`.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use util::state::AppState;
use validator::Validate;

use crate::response::ApiResponse;
use db::models::user::{
    ActiveModel as UserActiveModel, Column as UserColumn, Entity as UserEntity, Model as UserModel,
};

use super::common::{TestUserResponse, UpsertUserRequest};

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
    State(app): State<AppState>,
    Json(req): Json<UpsertUserRequest>,
) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(format!("Validation failed: {e}"))),
        )
            .into_response();
    }

    let db = app.db();
    let admin = req.admin.unwrap_or(false);

    match UserEntity::find()
        .filter(UserColumn::Username.eq(req.username.clone()))
        .one(db)
        .await
    {
        Ok(Some(existing)) => {
            // Update existing user
            let hash = UserModel::hash_password(&req.password);
            let mut am: UserActiveModel = existing.into();
            am.email = Set(req.email.clone());
            am.admin = Set(admin);
            am.password_hash = Set(hash);

            match am.update(db).await {
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
            match UserModel::create(db, &req.username, &req.email, &req.password, admin).await {
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
