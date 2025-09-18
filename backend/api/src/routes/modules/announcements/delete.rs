//! Announcement deletion handler.
//!
//! Provides an endpoint to delete an existing announcement.
//!
//! **Permissions:** Only users with the proper roles (e.g., lecturer/assistant) can delete announcements.

use crate::response::ApiResponse;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use db::models::announcements::Model as AnnouncementModel;
use util::state::AppState;

/// DELETE /api/modules/{module_id}/announcements/{announcement_id}
///
/// Deletes a single announcement by ID under the given module.
///
/// # AuthZ / AuthN
/// - Requires a valid `Bearer` token (JWT).
/// - Caller must be **lecturer** or **assistant_lecturer** on the module
///   (enforced by `require_lecturer_or_assistant_lecturer` on this route).
///
/// # Path Parameters
/// - `module_id` — ID of the parent module (used for route nesting and auth).
/// - `announcement_id` — ID of the announcement to delete.
///
/// # Behavior
/// - Deletion is **idempotent**: attempting to delete a non-existent announcement
///   will still return `200 OK` in this implementation (the driver returns
///   `rows_affected = 0`, which we do not currently treat as an error).
///
/// # Example cURL
/// ```bash
/// curl -X DELETE "https://your.api/api/modules/101/announcements/1234" \
///   -H "Authorization: Bearer <JWT>"
/// ```
///
/// # Responses
/// - `200 OK` — Always returned on successful DB call, even if the record didn’t exist.
/// - `401 UNAUTHORIZED` — Missing/invalid token.
/// - `403 FORBIDDEN` — Authenticated but not lecturer/assistant on this module.
/// - `500 INTERNAL SERVER ERROR` — Database error.
///
/// ## 200 OK — Example
/// ```json
/// {
///   "success": true,
///   "data": null,
///   "message": "Announcement deleted successfully"
/// }
/// ```
///
/// ## 403 Forbidden — Example
/// ```json
/// {
///   "success": false,
///   "message": "Forbidden"
/// }
/// ```
///
/// ## 500 Internal Server Error — Example
/// ```json
/// {
///   "success": false,
///   "message": "Failed to delete announcement: <database error details>"
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
            Json(ApiResponse::error(format!(
                "Failed to delete announcement: {}",
                err
            ))),
        ),
    }
}
