use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use util::state::AppState;

use crate::response::ApiResponse;
use db::models::user::{Column as UserColumn, Entity as UserEntity};

use super::common::TestUserResponse;
use crate::response::ApiResponse;
use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use services::service::Service;
use services::user::UserService;
use util::filters::FilterParam;

/// Query parameters for fetching a test user.
#[derive(Debug, Deserialize)]
pub struct GetUserParams {
    /// The unique username of the user to fetch.
    pub username: String,
}

/// GET `/api/test/users?username={username}`
///
/// Fetches a user by their username.
///
/// This endpoint is **only available in non-production environments**  
/// and exists for the sole purpose of E2E/integration tests.  
/// It must never be exposed in production.
///
/// # Query parameters
///
/// - `username` — The unique username of the user to fetch.
///
/// # Example request
///
/// ```bash
/// curl -X GET "http://localhost:3000/api/test/users?username=student42"
/// ```
///
/// # Response examples
///
/// ## 200 OK
/// ```json
/// {
///   "success": true,
///   "data": {
///     "id": 1,
///     "username": "student42",
///     "email": "student42@example.com",
///     "admin": false
///   },
///   "message": "OK"
/// }
/// ```
///
/// ## 404 Not Found
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "User not found"
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
pub async fn get_user(Query(params): Query<GetUserParams>) -> impl IntoResponse {
    match UserService::find_one(
        &vec![FilterParam::eq("username", params.username)],
        &vec![],
        None,
    )
    .await
    {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                TestUserResponse::from(user),
                "OK".to_string(),
            )),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("User not found".to_string())),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {e}"))),
        )
            .into_response(),
    }
}
