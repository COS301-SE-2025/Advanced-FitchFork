use axum::{response::IntoResponse, Json};
use crate::response::ApiResponse;

/// GET /example
///
/// Returns a simple confirmation message for the example route.
///
/// ### Response
/// - `200 OK`
///
/// ```json
/// {
///   "success": true,
///   "data": "Example index",
///   "message": "Fetched list"
/// }
/// ```
pub async fn index() -> impl IntoResponse {
    Json(ApiResponse::success("Example index", "Fetched list"))
}
