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

use axum::Router;
use crate::routes::{
    health::health_routes,
    example::example_routes,
    auth::auth_routes,
    users::users_routes,
    modules::modules_routes,
};

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
        .nest("/health", health_routes())
        .nest("/example", example_routes())
        .nest("/auth", auth_routes())
        .nest("/users", users_routes())
        .nest("/modules", modules_routes())
}
