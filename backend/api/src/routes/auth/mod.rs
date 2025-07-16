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
use get::{get_me, get_avatar, has_role_in_module};

// # Auth Routes Module
//
// This module defines and wires up routes under the `/api/auth` endpoint group.
//
// ## Structure
// - `post.rs` — POST handlers for authentication actions like registration, login, and password reset.
// - `get.rs` — GET handlers for retrieving authenticated user info, avatars, and role checks.
//
// ## Routes
// - `POST /auth/register` — Register a new user.
// - `POST /auth/login` — Authenticate an existing user.
// - `POST /auth/request-password-reset` — Initiate password reset process.
// - `POST /auth/verify-reset-token` — Validate a password reset token.
// - `POST /auth/reset-password` — Complete password reset.
// - `POST /auth/upload-profile-picture` — Upload a user profile picture.
// - `GET /auth/me` — Retrieve info about the currently authenticated user.
// - `GET /auth/avatar/{user_id}` — Retrieve a user's profile picture.
// - `GET /auth/has-role` — Check if the current user has a role in a module.
//
// ## Usage
// Use the `auth_routes()` function to mount all `/auth` endpoints under the main application router.

pub fn auth_routes() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/request-password-reset", post(request_password_reset))
        .route("/verify-reset-token", post(verify_reset_token))
        .route("/reset-password", post(reset_password))
        .route("/me", get(get_me))
        .route("/upload-profile-picture", post(upload_profile_picture))
        .route("/avatar/{user_id}", get(get_avatar))
        .route("/has-role", get(has_role_in_module))
    
}