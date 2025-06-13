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

use post::{register, login, request_password_reset, verify_reset_token, reset_password, upload_profile_picture};
use get::{get_me, get_own_avatar};

/// Builds the `/auth` route group, mapping HTTP methods to handlers.
///
/// - `POST /auth/register` → `register`
/// - `POST /auth/login` → `login`
/// - `POST /auth/request-password-reset` → `request_password_reset`
/// - `POST /auth/verify-reset-token` → `verify_reset_token`
/// - `POST /auth/reset-password` → `reset_password`
/// - `GET /auth/me` → `get_me`
///
/// # Returns
/// A configured `Router` instance to be nested in the main app.
pub fn auth_routes() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/request-password-reset", post(request_password_reset))
        .route("/verify-reset-token", post(verify_reset_token))
        .route("/reset-password", post(reset_password))
        .route("/me", get(get_me))
        .route("avatar/me", get(get_own_avatar))
        .route("/upload-profile-picture", post(upload_profile_picture))
}
