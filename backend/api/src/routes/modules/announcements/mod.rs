//! # Announcements Routes Module
//!
//! Defines routes for managing announcements under a module's `/announcements` endpoint.
//!
//! ## Routes
//! - `POST   /`                  → Create a new announcement (**lecturer or assistant lecturer** only).
//! - `GET    /`                  → Get a paginated list of announcements for the module.
//! - `GET    /{announcement_id}` → Get a single announcement by ID (includes only `id` & `username` of author).
//! - `PUT    /{announcement_id}` → Edit an existing announcement (**lecturer or assistant lecturer** only).
//! - `DELETE /{announcement_id}` → Delete an announcement (**lecturer or assistant lecturer** only).
//!
//! ## Access Control
//! - Write operations (`POST`, `PUT`, `DELETE`) require `require_lecturer_or_assistant_lecturer`.
//! - Read operations (`GET`) are open to any user assigned to the module (handled at parent route layer).

use axum::{middleware::from_fn_with_state, Router};
use util::state::AppState;

pub mod post;
pub mod get;
pub mod delete;
pub mod put;
pub mod common;

use axum::routing::{post, delete, put, get};
use post::create_announcement;
use delete::delete_announcement;
use put::edit_announcement;
use get::{get_announcements, get_announcement};
use crate::auth::guards::require_lecturer_or_assistant_lecturer;

/// Builds the `/announcements` route group for a specific module.
///
/// Routes:
/// - POST `/`                  → create announcement (lecturer or assistant lecturer only)
/// - GET `/`                   → list announcements
/// - GET `/{announcement_id}`  → get single announcement (with author id & username)
/// - PUT `/{announcement_id}`  → edit announcement (lecturer or assistant lecturer only)
/// - DELETE `/{announcement_id}` → delete announcement (lecturer or assistant lecturer only)
pub fn announcement_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/",post(create_announcement).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
        .route("/{announcement_id}",delete(delete_announcement).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
        .route("/{announcement_id}",put(edit_announcement).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
        .route("/", get(get_announcements))
        .route("/{announcement_id}", get(get_announcement))
}
