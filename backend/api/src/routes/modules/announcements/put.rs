//! Edit announcement handler.
//!
//! Provides an endpoint to update an existing announcement for a specific module.
//!
//! **Permissions:** Only authorized users (lecturer/assistant) can edit announcements.

use crate::{response::ApiResponse, routes::modules::announcements::common::AnnouncementRequest};
use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use services::announcement::{AnnouncementService, UpdateAnnouncement};
use services::service::Service;

/// PUT /api/modules/{module_id}/announcements/{announcement_id}
///
/// Updates a single announcement under the given module.
///
/// # AuthZ / AuthN
/// - Requires a valid `Bearer` token (JWT).
/// - Caller must be **lecturer** or **assistant_lecturer** on the module
///   (enforced by `require_lecturer_or_assistant_lecturer` on this route).
///
/// # Path Parameters
/// - `module_id` — ID of the parent module (used for nesting & auth).
/// - `announcement_id` — ID of the announcement to update.
///
/// # Request Body
/// JSON matching `AnnouncementRequest`:
/// ```json
/// {
///   "title": "New title (or empty string to keep existing)",
///   "body": "Updated body (or empty string to keep existing)",
///   "pinned": true
/// }
/// ```
///
/// **Partial update semantics:**
/// - `title`: if empty string `""`, the existing title is kept.
/// - `body`: if empty string `""`, the existing body is kept.
/// - `pinned`: always updated to the provided boolean.
///
/// # Example cURL
/// ```bash
/// curl -X PUT "https://your.api/api/modules/101/announcements/1234" \
///   -H "Authorization: Bearer <JWT>" \
///   -H "Content-Type: application/json" \
///   -d '{"title":"Exam venue update","body":"**Hall A** instead of Hall B.","pinned":false}'
/// ```
///
/// # Responses
/// - `200 OK` — Returns the updated announcement.
/// - `401 UNAUTHORIZED` — Missing/invalid token.
/// - `403 FORBIDDEN` — Authenticated but not lecturer/assistant on this module.
/// - `422 UNPROCESSABLE ENTITY` — Malformed/invalid JSON for `AnnouncementRequest`.
/// - `500 INTERNAL SERVER ERROR` — Database error.
///
/// ## 200 OK — Example
/// ```json
/// {
///   "success": true,
///   "data": {
///     "id": 1234,
///     "module_id": 101,
///     "user_id": 5,
///     "title": "Exam venue update",
///     "body": "**Hall A** instead of Hall B.",
///     "pinned": false,
///     "created_at": "2025-08-16T12:00:00Z",
///     "updated_at": "2025-08-16T12:30:00Z"
///   },
///   "message": "Announcement updated successfully"
/// }
/// ```
///
/// ## 422 Unprocessable Entity — Example
/// ```json
/// {
///   "success": false,
///   "message": "Unprocessable Entity"
/// }
/// ```
///
/// ## 500 Internal Server Error — Example
/// ```json
/// {
///   "success": false,
///   "message": "Failed to update announcement"
/// }
/// ```
pub async fn edit_announcement(
    Path((_, announcement_id)): Path<(i64, i64)>,
    Json(req): Json<AnnouncementRequest>,
) -> impl IntoResponse {
    match AnnouncementService::update(UpdateAnnouncement {
        id: announcement_id,
        title: Some(req.title),
        body: Some(req.body),
        pinned: Some(req.pinned),
    })
    .await
    {
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
