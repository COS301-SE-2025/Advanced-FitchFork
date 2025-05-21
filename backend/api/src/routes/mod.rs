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
//!
//! ## Usage
//! Call `routes()` from your main application to initialize all top-level routes.

pub mod health;
pub mod example;

use axum::Router;
use crate::routes::{health::health_routes, example::example_routes};

/// Builds the complete application router.
///
/// This nests sub-routes under the appropriate base paths:
/// - `/health` → health check endpoint
/// - `/example` → example feature endpoints
///
/// # Returns
/// An Axum `Router` ready to be passed into the main app.
pub fn routes() -> Router {
    Router::new()
        .nest("/health", health_routes())
        .nest("/example", example_routes())
}
