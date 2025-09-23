//! HTTP route entry point for `/api/...`.
//!
//! This module defines all HTTP entry points under the `/api` namespace.
//! Routes are organized by domain (e.g., authentication, users, modules, health),
//! each protected via appropriate access control middleware.  
//!
//! Route groups include:
//! - `/health` → Health check endpoint (public)
//! - `/auth` → Authentication endpoints (login, token handling, public)
//! - `/users` → User management endpoints (admin-only)
//! - `/modules` → Module management, personnel, and assignments (authenticated users)
//! - `/me` → User-specific endpoints (announcements, tickets, assignments)

use crate::auth::guards::{allow_admin, allow_authenticated};
use crate::routes::auth::get::get_avatar;
use crate::routes::me::me_routes;
use crate::routes::{
    auth::auth_routes, health::health_routes, modules::modules_routes, system::system_routes,
    test::test_routes, users::users_routes,
};
use axum::{Router, middleware::from_fn, routing::get};
use util::{config, state::AppState};

pub mod auth;
pub mod common;
pub mod health;
pub mod me;
pub mod modules;
pub mod system;
pub mod test;
pub mod users;

/// Builds the complete application router for all HTTP endpoints.
///
/// The returned router has `AppState` as its state type and mounts
/// all core API routes under their respective base paths.  
///
/// # Route Structure:
/// - `/health` → Health check endpoint (no authentication required).
/// - `/auth` → Authentication endpoints (login, refresh, etc.).
/// - `/users` → User management (restricted to admins via `require_admin` middleware).
/// - `/users/{user_id}/avatar` → Publicly accessible avatar retrieval.
/// - `/modules` → Module CRUD, personnel management, and assignments (requires authentication).
/// - `/me` → User-specific endpoints (announcements, tickets, assignments, etc.)
/// - `/test` → Development/test-only routes (mounted only if `env != production`).
///
/// The `/test` route group is mounted **here** instead of in `main` to:
/// 1. Keep `main` focused on server startup logic only.
/// 2. Avoid changing the `Router` type after construction, which can cause trait bound issues.
/// 3. Ensure that all route registration logic is centralized in one place.
pub fn routes(app_state: AppState) -> Router<AppState> {
    let mut router: Router<AppState> = Router::new()
        .nest("/health", health_routes())
        .nest("/auth", auth_routes())
        .nest("/users", users_routes().route_layer(from_fn(allow_admin)))
        .route("/users/{user_id}/avatar", get(get_avatar))
        .nest(
            "/modules",
            modules_routes(app_state.clone()).route_layer(from_fn(allow_authenticated)),
        )
        .nest("/me", me_routes().route_layer(from_fn(allow_authenticated)))
        .nest("/system", system_routes())
        .with_state(app_state.clone());

    // Conditionally mount the `/test` route group if *not* in production.
    //
    // This keeps development and test-only APIs out of the production environment,
    // but still makes them available in `development` or `test` modes.
    let env = config::env().to_lowercase();
    if env != "production" {
        router = router.nest("/test", test_routes(app_state.clone()));
        tracing::info!("[dev/test] Mounted /test routes (env = {env})");
    } else {
        tracing::info!("[prod] Skipping /test routes");
    }

    router
}
