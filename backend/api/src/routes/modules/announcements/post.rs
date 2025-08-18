//! Create announcement handler.
//!
//! Provides an endpoint to create a new announcement for a specific module.
//!
//! **Permissions:** Only authorized users (lecturer/assistant) can create announcements.

use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Extension, Json};
use util::state::AppState;
use crate::{auth::AuthUser, response::ApiResponse, routes::modules::announcements::common::AnnouncementRequest};
use db::models::announcements::Model as AnnouncementModel;

/// Creates a new announcement for a given module.
///
/// **Endpoint:** `POST /modules/{module_id}/announcements`  
/// **Permissions:** Only authorized users (lecturer/assistant) can create an announcement.
///
/// ### Path parameters
/// - `module_id` → ID of the module to create the announcement for
///
/// ### Request body
/// ```json
/// {
///   "title": "Announcement title",
///   "body": "Announcement body",
///   "pinned": true // optional, defaults to false
/// }
/// ```
///
/// ### Responses
/// - `200 OK` → Announcement created successfully
/// ```json
/// {
///   "success": true,
///   "data": { /* Announcement object */ },
///   "message": "Announcement created successfully"
/// }
/// ```
/// - `500 Internal Server Error` → Failed to create announcement
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Failed to create announcement: <error details>"
/// }
/// ```
pub async fn create_announcement(
    State(app_state): State<AppState>,
    Path(module_id): Path<i64>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Json(req): Json<AnnouncementRequest>,
) -> impl IntoResponse {
    let db = app_state.db();
    let user_id = claims.sub;

    match AnnouncementModel::create(db, module_id, user_id, &req.title, &req.body, req.pinned).await {
        Ok(announcement) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                announcement,
                "Announcement created successfully",
            )),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                format!("Failed to create announcement: {}", err),
            )),
        ),
    }
}
