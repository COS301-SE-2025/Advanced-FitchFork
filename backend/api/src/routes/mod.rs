//! # Application Routes
//!
//! This module defines the top-level routing configuration for the API.
//!
//! Each submodule is responsible for its own route group (e.g., `/health`),
//! and those groups are composed together here.
//!
//! ## Structure
//! - `health.rs`: Contains the `/health` route for basic uptime checks.
//! - `auth/`: A folder containing `post.rs` for the `/auth` API group.
//! - `users/`: A folder containing `get.rs` for the `/users` API group.
//! - `modules/`: A folder containing `post.rs` for the `/modules` API group.
//!
//! ## Usage
//! Call `routes()` from your main application to initialize all top-level routes.

use axum::{middleware::from_fn, Router};
use crate::routes::{auth::auth_routes, health::health_routes, modules::modules_routes, users::users_routes};
use crate::auth::guards::{require_admin, require_authenticated};
use sea_orm::DatabaseConnection;

pub mod health;
pub mod auth;
pub mod users;
pub mod modules;
pub mod common;

/// Builds the complete application router.
///
/// This nests sub-routes under the appropriate base paths:
/// - `/health` → health check endpoint
/// - `/auth` → authentication endpoints
/// - `/users` → user management endpoints
/// - `/modules` → module management endpoints
/// - `/assignments` → assignment management endpoints
///
/// # Returns
/// An Axum `Router` ready to be passed into the main app.
pub fn routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .with_state(db.clone())
        .nest("/health", health_routes())
        .nest("/auth", auth_routes(db.clone()))
        .nest("/users", users_routes(db.clone()).route_layer(from_fn(require_admin)))
        .nest("/modules", modules_routes(db.clone()).route_layer(from_fn(require_authenticated)))
}