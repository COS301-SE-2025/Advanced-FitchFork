//! Edit announcement handler.
//!
//! Provides an endpoint to update an existing announcement for a specific module.
//!
//! **Permissions:** Only authorized users (lecturer/assistant) can edit announcements.

use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use db::models::announcements::Model as AnnouncementModel;
use crate::{response::ApiResponse, routes::modules::announcements::common::AnnouncementRequest};
use util::state::AppState;

/// Updates an existing announcement.
///
/// **Endpoint:** `PUT /modules/{module_id}/announcements/{announcement_id}`  
/// **Permissions:** Only authorized users (lecturer/assistant) can update the announcement.
///
/// ### Path parameters
/// - `module_id` → ID of the module (used for consistency, may be used for permission checks)
/// - `announcement_id` → ID of the announcement to update
///
/// ### Request body
/// ```json
/// {
///   "title": "Updated announcement title",
///   "body": "Updated announcement body",
///   "pinned": true
/// }
/// ```
///
/// ### Responses
/// - `200 OK` → Announcement updated successfully
/// ```json
/// {
///   "success": true,
///   "data": { /* Updated announcement object */ },
///   "message": "Announcement updated successfully"
/// }
/// ```
/// - `500 Internal Server Error` → Failed to update announcement
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Failed to update announcement"
/// }
/// ```
pub async fn edit_announcement(
    State(app_state): State<AppState>,
    Path((_, announcement_id)): Path<(i64, i64)>,
    Json(req): Json<AnnouncementRequest>,
) -> impl IntoResponse {
    let db = app_state.db();

    match AnnouncementModel::update(db, announcement_id, &req.title, &req.body, req.pinned).await {
        Ok(updated_announcement) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                updated_announcement,
                "Announcement updated successfully",
            )),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("Failed to update announcement")),
        ),
    }
}
