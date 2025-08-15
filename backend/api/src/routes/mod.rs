use crate::auth::guards::{require_admin, require_authenticated};
use crate::routes::{auth::auth_routes, health::health_routes, modules::modules_routes, users::users_routes};
use axum::{middleware::from_fn, Router};

pub mod auth;
pub mod common;
pub mod health;
pub mod modules;
pub mod users;

/// Builds the complete application router.
///
/// This nests sub-routes under the appropriate base paths:
/// - `/health` → health check endpoint
/// - `/auth` → authentication endpoints (login, token handling)
/// - `/users` → user management (admin-only)
/// - `/modules` → module CRUD, personnel management, assignments (authenticated users)
/// - `/plagiarism` → plagiarism detection and results
/// - `/ws` → WebSocket topics (real-time communication via guarded topic namespaces)
///
/// # Returns
/// An Axum `Router<AppState>` with all route groups and middleware applied.
pub fn routes() -> Router {
    Router::new()
        .nest("/health", health_routes())
        .nest("/auth", auth_routes())
        .nest("/users", users_routes().route_layer(from_fn(require_admin)))
        .nest("/modules", modules_routes().route_layer(from_fn(require_authenticated)))
}