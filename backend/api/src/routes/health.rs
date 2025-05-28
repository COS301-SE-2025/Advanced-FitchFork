use axum::{Router, routing::get, Json, response::IntoResponse};
use crate::response::ApiResponse;

/// Builds the `/health` route group.
///
/// This includes a single `GET /health` endpoint that returns a basic success message.
/// Useful for uptime checks, load balancers, or deployment health monitoring.
///
/// # Returns
/// An Axum `Router` with the `GET /health` route configured.
pub fn health_routes() -> Router {
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

#[cfg(test)]
mod tests {
    use super::health_check;
    use axum::response::IntoResponse;
    use axum::body::to_bytes;
    use serde_json::Value;

    /// Unit test for `health_check` handler.
    ///
    /// Asserts that the JSON response matches the expected structure and values.
    #[tokio::test]
    async fn health_check_returns_ok_json() {
        let response = health_check().await.into_response();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["data"], "OK");
        assert_eq!(json["message"], "Health check passed");
    }
}
