//! # Announcement Routes Module
//!
//! Defines and wires up routes for the `/api/modules/{module_id}/announcements` endpoint group.
//!
//! ## Structure
//! - `post.rs` — POST handlers (e.g., create announcement)
//! - `get.rs` — GET handlers (e.g., list announcements)
//! - `put.rs` — PUT handlers (e.g., edit announcement)
//! - `delete.rs` — DELETE handlers (e.g., remove announcement)
//! - `common.rs` — shared helpers and utilities
//!
//! ## Usage
//! Call `announcement_routes(app_state)` to get a configured `Router` for announcements
//! to be mounted under a module in the main app.

use axum::{middleware::from_fn_with_state, Router};
use util::state::AppState;
use axum::routing::{post, delete, put, get};
use post::create_announcement;
use delete::delete_announcement;
use put::edit_announcement;
use get::{get_announcements, get_announcement};
use crate::auth::guards::require_lecturer_or_assistant_lecturer;

pub mod post;
pub mod get;
pub mod delete;
pub mod put;
pub mod common;



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
