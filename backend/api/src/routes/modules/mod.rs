//! # modules Routes Module
//!
//! This module defines and wires up routes for the `/modules` endpoint group.
//!
//! ## Structure
//! - `post.rs` — POST handlers (e.g., create module, assign lecturers)
//!
//! ## Usage
//! The `modules_routes()` function returns a `Router` which is nested under `/modules` in the main application.

pub mod post;
pub mod delete;

use axum::{
    Router,
    routing::{post, delete},
};
use crate::auth::guards::require_admin;
use post::{create, assign_lecturers};
use delete::{remove_lecturers};
/// Builds the `/modules` route group, mapping HTTP methods to handlers.
///
/// - `POST /` → `create` (admin only)
/// - `POST /:module_id/lecturers` → `assign_lecturers` (admin only)
///
/// # Returns
/// A configured `Router` instance to be nested in the main app.
pub fn modules_routes() -> Router {
    Router::new()
        .route("/", post(create))
        .route("/:module_id/lecturers", post(assign_lecturers))
        .route("/:module_id/lecturers", delete(remove_lecturers))
        .route_layer(axum::middleware::from_fn(require_admin))
}
