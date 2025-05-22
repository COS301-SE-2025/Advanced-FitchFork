//! # Example Routes Module
//!
//! This module defines and wires up routes for the `/example` endpoint group.
//!
//! ## Structure
//! - `get.rs` — GET handlers (e.g., index list)
//! - `post.rs` — POST handlers (e.g., create resource)
//! - `delete.rs` — DELETE handlers (e.g., delete by ID, protected by middleware)
//!
//! ## Middleware
//! The DELETE route is protected using `dummy_auth` middleware as a placeholder for real authentication.
//!
//! ## Usage
//! The `example_routes()` function returns a `Router` which is nested under `/example` in the main application.

pub mod get;
pub mod post;
pub mod delete;

use axum::{
    Router,
    routing::{get, post, delete},
    middleware,
};
use crate::auth::middleware::dummy_auth;

use get::index;
use post::create;
use delete::delete_example;

/// Builds the `/example` route group, mapping HTTP methods to handlers.
///
/// - `GET /example` → `index`
/// - `POST /example` → `create`
/// - `DELETE /example/:id` → `delete_example` (requires `dummy_auth`)
///
/// # Returns
/// A configured `Router` instance to be nested in the main app.
pub fn example_routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/", post(create))
        .route("/:id", delete(delete_example).layer(middleware::from_fn(dummy_auth)))
}
