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

pub mod get;
pub mod put;
pub mod delete;

use axum::{
    Router,
    routing::{get, put, delete},
};
use crate::auth::guards::require_admin;
use get::list_users;
use get::{get_user_modules, get_user};
use put::update_user;
use delete::delete_user;
use crate::routes::users::put::upload_user_avatar;

/// Builds the `/users` route group, mapping HTTP methods to handlers.
///
/// - `GET /users` → `list_users` (admin only)
/// - `GET /users/{id}/modules` → `get_user_modules` (admin only)
/// - `PUT /users/{id}` → `update_user` (admin only)
/// - `DELETE /users/{id}` → `delete_user` (admin only)
///
/// # Returns
/// A configured `Router` instance to be nested in the main app.
pub fn users_routes() -> Router {
    Router::new()
        .route("/", get(list_users))
        .route("/{id}/modules", get(get_user_modules))
        .route("/{id}", get(get_user))
        .route("/{id}", put(update_user))
        .route("/{id}", delete(delete_user))
        .route("/{id}/avatar", put(upload_user_avatar))
        .route_layer(axum::middleware::from_fn(require_admin))
}