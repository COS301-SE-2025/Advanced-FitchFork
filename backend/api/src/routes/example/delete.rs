use axum::{extract::Path, Json, response::IntoResponse};
use crate::response::ApiResponse;

/// DELETE /example/:id
///
/// Deletes an example resource by ID. This route is protected by the `dummy_auth` middleware.
///
/// ### Path Parameter
/// - `id`: The ID of the example resource to delete
///
/// ### Response
/// - `200 OK`
///
/// ```json
/// {
///   "success": true,
///   "data": "Deleted item 42",
///   "message": "Resource deleted"
/// }
/// ```
pub async fn delete_example(Path(id): Path<u32>) -> impl IntoResponse {
    Json(ApiResponse::success(format!("Deleted item {}", id), "Resource deleted"))
}
