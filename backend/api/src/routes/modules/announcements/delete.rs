//! Announcement deletion handler.
//!
//! Provides an endpoint to delete an existing announcement.
//!
//! **Permissions:** Only users with the proper roles (e.g., lecturer/assistant) can delete announcements.

use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use db::models::announcements::Model as AnnouncementModel;
use util::state::AppState;
use crate::response::ApiResponse;

/// Deletes an existing announcement.
///
/// **Endpoint:** `DELETE /announcements/{announcement_id}`  
/// **Permissions:** Only authorized users (lecturer/assistant) can delete an announcement.
///
/// ### Path parameters
/// - `announcement_id` → ID of the announcement to be deleted
///
/// ### Responses
/// - `200 OK` → Announcement deleted successfully
/// ```json
/// {
///   "success": true,
///   "data": {},
///   "message": "Announcement deleted successfully"
/// }
/// ```
/// - `500 Internal Server Error` → Failed to delete the announcement
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Failed to delete announcement: <error details>"
/// }
/// ```
pub async fn delete_announcement(
    State(app_state): State<AppState>,
    Path((_, announcement_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let db = app_state.db();
    match AnnouncementModel::delete(db, announcement_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                (),
                "Announcement deleted successfully",
            )),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                format!("Failed to delete announcement: {}", err),
            )),
        ),
    }
}
