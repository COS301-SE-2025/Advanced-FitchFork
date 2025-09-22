//! # Users Routes Module
//!
//! This module defines and wires up routes for the `/api/users` endpoint group.
//!
//! ## Structure
//! - `get.rs` — GET handlers (e.g., list users)
//! - `put.rs` — PUT handlers (e.g., update user)
//! - `delete.rs` — DELETE handlers (e.g., delete user)
//!
//! ## Middleware
//! The GET, PUT, and DELETE routes are protected using `require_admin` middleware.
//!
//! ## Usage
//! The `users_routes()` function returns a `Router` which is nested under `/users` in the main application.

use crate::routes::users::put::upload_avatar;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use delete::delete_user;
use get::{get_user, get_user_modules, list_users};
use post::{bulk_create_users, create_user};
use put::update_user;
use util::state::AppState;

pub mod common;
pub mod delete;
pub mod get;
pub mod post;
pub mod put;

/// Builds the `/users` route group, mapping HTTP methods to handlers.
///
/// - `GET /users` → `list_users` (admin only)
/// - `POST /users` → `create_user` (admin only)
/// - `POST /users/bulk` → `bulk_create_users` (admin only)
/// - `GET /users/{user_id}/modules` → `get_user_modules` (admin only)
/// - `GET /users/{user_id}` → `get_user` (admin only)
/// - `PUT /users/{user_id}` → `update_user` (admin only)
/// - `DELETE /users/{user_id}` → `delete_user` (admin only)
/// - `PUT /users/{user_id}/avatar` → `upload_avatar` (admin only)
///
/// # Returns
/// A configured `Router` instance to be nested in the main app.

pub fn users_routes() -> Router {
    Router::new()
        .route("/", get(list_users))
        .route("/", post(create_user))
        .route("/bulk", post(bulk_create_users))
        .route("/{user_id}/modules", get(get_user_modules))
        .route("/{user_id}", get(get_user))
        .route("/{user_id}", put(update_user))
        .route("/{user_id}", delete(delete_user))
        .route("/{user_id}/avatar", put(upload_avatar))
}
