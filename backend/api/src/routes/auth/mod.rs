//! # auth Routes Module
//!
//! This module defines and wires up routes for the `/auth` endpoint group.
//!
//! ## Structure
//! - `post.rs` — POST handlers (e.g., register)
//! - `get.rs` — GET handlers (e.g., current user info)
//!
//! ## Usage
//! The `auth_routes()` function returns a `Router` which is nested under `/auth` in the main application.

pub mod post;
pub mod get;

use axum::{
    Router,
    routing::{post, get},
};

use post::{register, login, request_password_reset};
use get::get_me;

/// Builds the `/auth` route group, mapping HTTP methods to handlers.
///
/// - `POST /auth/register` → `register`
/// - `POST /auth/login` → `login`
/// - `POST /auth/request-password-reset` → `request_password_reset`
/// - `GET /auth/me` → `get_me`
///
/// # Returns
/// A configured `Router` instance to be nested in the main app.
pub fn auth_routes() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/request-password-reset", post(request_password_reset))
        // .route("/verify-reset-token", post(verify_reset))
        // .route("/reset-password", post(reset_password))
        .route("/me", get(get_me))
}
