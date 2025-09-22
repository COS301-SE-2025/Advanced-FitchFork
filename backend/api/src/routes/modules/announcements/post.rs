//! Create announcement handler.
//!
//! Provides an endpoint to create a new announcement for a specific module.
//!
//! **Permissions:** Only authorized users (lecturer/assistant) can create announcements.

use crate::{
    auth::AuthUser, response::ApiResponse,
    routes::modules::announcements::common::AnnouncementRequest,
};
use axum::{Extension, Json, extract::Path, http::StatusCode, response::IntoResponse};
use services::announcement::{AnnouncementService, CreateAnnouncement};
use services::service::Service;

/// POST /api/modules/{module_id}/announcements
///
/// Creates a new announcement for the specified module.
///
/// # AuthZ / AuthN
/// - Requires a valid `Bearer` token (JWT).
/// - Caller must be **lecturer** or **assistant_lecturer** on the target module
///   (enforced by `require_lecturer_or_assistant_lecturer` route layer).
///
/// # Path Parameters
/// - `module_id` — ID of the module to create the announcement under.
///
/// # Request Body
/// JSON matching `AnnouncementRequest`:
/// ```json
/// {
///   "title": "Exam Schedule",
///   "body": "The exam will be held next **Friday** at 09:00.",
///   "pinned": true
/// }
/// ```
///
/// # Example cURL
/// ```bash
/// curl -X POST "https://your.api/api/modules/101/announcements" \
///   -H "Authorization: Bearer <JWT>" \
///   -H "Content-Type: application/json" \
///   -d '{
///         "title": "Exam Schedule",
///         "body": "The exam will be held next **Friday** at 09:00.",
///         "pinned": true
///       }'
/// ```
///
/// # Responses
/// - `200 OK` — Announcement created successfully. Returns the created record.
/// - `400 BAD REQUEST` — Malformed JSON.
/// - `401 UNAUTHORIZED` — Missing/invalid token.
/// - `403 FORBIDDEN` — Authenticated but not lecturer/assistant on this module.
/// - `422 UNPROCESSABLE ENTITY` — JSON is valid but required fields missing/invalid.
/// - `500 INTERNAL SERVER ERROR` — Database error.
///
/// ## 200 OK — Example
/// ```json
/// {
///   "success": true,
///   "data": {
///     "id": 1234,
///     "module_id": 101,
///     "user_id": 55,
///     "title": "Exam Schedule",
///     "body": "The exam will be held next **Friday** at 09:00.",
///     "pinned": true,
///     "created_at": "2025-02-10T12:34:56Z",
///     "updated_at": "2025-02-10T12:34:56Z"
///   },
///   "message": "Announcement created successfully"
/// }
/// ```
///
/// ## 400 Bad Request — Example (invalid JSON)
/// ```json
/// {
///   "success": false,
///   "message": "invalid JSON body"
/// }
/// ```
///
/// ## 422 Unprocessable Entity — Example (missing fields)
/// ```json
/// {
///   "success": false,
///   "message": "Unprocessable Entity"
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
///   "message": "Failed to create announcement: database error detail ..."
/// }
/// ```
pub async fn create_announcement(
    Path(module_id): Path<i64>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Json(req): Json<AnnouncementRequest>,
) -> impl IntoResponse {
    match AnnouncementService::create(CreateAnnouncement {
        module_id,
        user_id: claims.sub,
        title: req.title,
        body: req.body,
        pinned: req.pinned,
    })
    .await
    {
        Ok(announcement) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                announcement,
                "Announcement created successfully",
            )),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!(
                "Failed to create announcement: {}",
                err
            ))),
        ),
    }
}
