//! # auth Routes Module
//!
//! This module defines and wires up routes for the `/auth` endpoint group.
//!
//! ## Structure
//! - `post.rs` — POST handlers (e.g., register)
//!
//! ## Usage
//! The `auth_routes()` function returns a `Router` which is nested under `/auth` in the main application.

pub mod post;

use axum::{
    Router,
    routing::{post}
};

use post::register;
use post::login;

/// Builds the `/auth` route group, mapping HTTP methods to handlers.
///
/// - `POST /auth/register` → `register`
/// - `POST /auth/login` → `login`
///
/// # Returns
/// A configured `Router` instance to be nested in the main app.
pub fn auth_routes() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}