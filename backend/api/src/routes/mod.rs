//! # Application Routes
//!
//! This module defines the top-level routing configuration for the API.
//!
//! Each submodule is responsible for its own route group (e.g., `/health`, `/example`),
//! and those groups are composed together here.
//!
//! ## Structure
//! - `health.rs`: Contains the `/health` route for basic uptime checks.
//! - `example/`: A folder containing `get.rs`, `post.rs`, `delete.rs`, etc. for the `/example` API group.
//! - `auth/`: A folder containing `post.rs` for the `/auth` API group.
//! - `users/`: A folder containing `get.rs` for the `/users` API group.
//! - `modules/`: A folder containing `post.rs` for the `/modules` API group.
//!
//! ## Usage
//! Call `routes()` from your main application to initialize all top-level routes.

pub mod health;
pub mod example;
pub mod auth;
pub mod users;
pub mod modules;
pub mod common;

use axum::Router;
use axum::middleware::from_fn;

use crate::routes::{
    auth::auth_routes,
    example::example_routes,
    health::health_routes,
    modules::modules_routes,
    users::users_routes,
};
use crate::auth::guards::{require_admin, require_authenticated};

/// Builds the complete application router.
///
/// This nests sub-routes under the appropriate base paths:
/// - `/health` → health check endpoint
/// - `/example` → example feature endpoints 
/// - `/auth` → authentication endpoints
/// - `/users` → user management endpoints
/// - `/modules` → module management endpoints
/// - `/assignments` → assignment management endpoints
///
/// # Returns
/// An Axum `Router` ready to be passed into the main app.
pub fn routes() -> Router {
    Router::new()
        .nest("/health", health_routes()) // No protection
        .nest("/example", example_routes()) // Public or protected as needed inside
        .nest("/auth", auth_routes()) // No auth required for login/register
        .nest("/users", users_routes().route_layer(from_fn(require_admin))) // Admin-only access
        .nest("/modules", modules_routes().route_layer(from_fn(require_authenticated))) // All module routes require auth
}
