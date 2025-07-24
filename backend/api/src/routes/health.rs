use axum::{Router, routing::get, Json, response::IntoResponse};
use crate::response::ApiResponse;
use sea_orm::DatabaseConnection;

/// Builds the `/health` route group.
///
/// This includes a single `GET /health` endpoint that returns a basic success message.
/// Useful for uptime checks, load balancers, or deployment health monitoring.
///
/// # Returns
/// An Axum `Router` with the `GET /health` route configured.
pub fn health_routes() -> Router<DatabaseConnection> {
    Router::new().route("/", get(health_check))
}

/// GET /health
///
/// Returns a simple success response to indicate the API is running.
/// This is often used for uptime checks or health probes.
///
/// ### Response
/// - `200 OK`
///
/// ```json
/// {
///   "success": true,
///   "data": "OK",
///   "message": "Health check passed"
/// }
/// ```
async fn health_check() -> impl IntoResponse {
    Json(ApiResponse::success("OK", "Health check passed"))
}