//! # modules Routes Module
//!
//! This module defines and wires up routes for the `/modules` endpoint group.
//!
//! ## Structure
//! - `post.rs` — POST handlers (e.g., register)
//!
//! ## Usage
//! The `modules_routes()` function returns a `Router` which is nested under `/modules` in the main application.

pub mod post;
pub mod assignments;
use assignments::assignment_routes;
use axum::{
    Router,
    routing::post,
};
use crate::auth::guards::require_admin;
use post::create;

/// Builds the `/modules` route group, mapping HTTP methods to handlers.
///
/// - `POST /` → `create` (admin only)
///
/// # Returns
/// A configured `Router` instance to be nested in the main app.
pub fn modules_routes() -> Router {
    Router::new()
        .route("/", post(create))
        .nest("/:module_id/assignments", assignment_routes())
        .route_layer(axum::middleware::from_fn(require_admin))
}