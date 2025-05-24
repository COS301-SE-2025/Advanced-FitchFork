use axum::{response::IntoResponse, extract::Extension, Json};
use crate::response::ApiResponse;
use crate::auth::claims::AuthUser;

/// GET /example
///
/// Public route for basic testing. No JWT required.
/// Returns a static confirmation message.
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

/// GET /example/auth
///
/// Protected route that requires a valid JWT token with standard claims.
/// Extracts the JWT claims using [`AuthUser`] and confirms the authenticated user ID.
///
/// ### JWT Claims Used
/// - `sub`: user ID
/// - `admin`: (ignored)
/// - `exp`: expiration check enforced by middleware
///
/// ### Requires
/// - `require_authenticated` middleware
///
/// ### Response
/// - `200 OK`
///
/// ```json
/// {
///   "success": true,
///   "data": "Test Get Route Auth",
///   "message": "Well done you authenticated, good boy user with id: 123"
/// }
/// ```
pub async fn test_get_route_auth(Extension(AuthUser(claims)): Extension<AuthUser>) -> impl IntoResponse {
    let message = format!("Well done you authenticated, good boy user with id: {}", claims.sub);
    Json(ApiResponse::success("Test Get Route Auth", message))
}

/// GET /example/admin
///
/// Admin-only route protected by JWT and an `admin: true` claim.
/// Confirms that the authenticated user has admin rights and displays their user ID.
///
/// ### JWT Claims Used
/// - `sub`: user ID
/// - `admin`: must be `true`
/// - `exp`: expiration check enforced by middleware
///
/// ### Requires
/// - `require_admin` middleware
///
/// ### Response
/// - `200 OK`
///
/// ```json
/// {
///   "success": true,
///   "data": "Test Get Route Admin",
///   "message": "Well done you are an Admin, good boy. Your user ID is: 123"
/// }
/// ```
pub async fn test_get_route_admin(Extension(AuthUser(claims)): Extension<AuthUser>) -> impl IntoResponse {
    let message = format!(
        "Well done you are an Admin, good boy. Your user ID is: {}",
        claims.sub
    );
    Json(ApiResponse::success("Test Get Route Admin", message))
}

/// GET /example/admin-auth
///
/// Redundant example route using **both** `require_authenticated` and `require_admin` middleware layers.
/// Demonstrates how multiple middleware can be composed for learning purposes.
///
/// ### JWT Claims Used
/// - `sub`: user ID
/// - `admin`: must be `true`
/// - `exp`: expiration check enforced by middleware
///
/// ### Requires
/// - `require_authenticated` middleware
/// - `require_admin` middleware
///
/// ### Response
/// - `200 OK`
///
/// ```json
/// {
///   "success": true,
///   "data": "Test Get Route Admin + Auth",
///   "message": "You passed both auth layers. Welcome, user ID 123"
/// }
/// ```
pub async fn test_get_route_admin_double_protected(Extension(AuthUser(claims)): Extension<AuthUser>) -> impl IntoResponse {
    let message = format!(
        "You passed both auth layers. Welcome, user ID {}",
        claims.sub
    );
    Json(ApiResponse::success("Test Get Route Admin + Auth", message))
}