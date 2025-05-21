use axum::{response::IntoResponse, Json};
use crate::response::ApiResponse;

/// POST /example
///
/// Creates a new example resource. This is a demonstration endpoint with no actual persistence.
///
/// ### Request Body
/// ```json
/// {
///   "name": "string",
///   "value": "any"
/// }
/// ```
///
/// ### Response
/// - `201 Created`
///
/// ```json
/// {
///   "success": true,
///   "data": "Created",
///   "message": "Resource created"
/// }
/// ```
pub async fn create() -> impl IntoResponse {
    Json(ApiResponse::success("Created", "Resource created"))
}
