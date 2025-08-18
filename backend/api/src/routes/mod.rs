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

use crate::auth::guards::{require_admin, require_authenticated};
use crate::routes::me::my_routes;
use crate::routes::{auth::auth_routes, health::health_routes, modules::modules_routes, users::users_routes};
use axum::{middleware::from_fn, Router};
use util::state::AppState;

pub mod auth;
pub mod common;
pub mod health;
pub mod modules;
pub mod users;
pub mod me;

/// Builds the complete application router.
///
/// This nests sub-routes under the appropriate base paths:
/// - `/health` → health check endpoint
/// - `/auth` → authentication endpoints (login, token handling)
/// - `/users` → user management (admin-only)
/// - `/modules` → module CRUD, personnel management, assignments (authenticated users)
///
/// # Returns
/// An Axum `Router<AppState>` with all route groups and middleware applied.
pub fn routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/health", health_routes())
        .nest("/auth", auth_routes())
        .nest("/users", users_routes().route_layer(from_fn(require_admin)))
        .nest("/modules", modules_routes(app_state.clone()).route_layer(from_fn(require_authenticated)))
        .nest("/me", my_routes(app_state.clone()).route_layer(from_fn(require_authenticated)))
}