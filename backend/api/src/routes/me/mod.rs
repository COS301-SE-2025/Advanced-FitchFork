//! # My Routes Module
//!
//! Defines and wires up routes for user-specific endpoints, such as announcements, tickets, and assignments.
//!
//! ## Structure
//! - `announcements.rs` — GET handlers for fetching the user's announcements
//! - `assignments.rs` — GET handlers for fetching the user's assignments
//! - `tickets.rs` — GET handlers for fetching the user's tickets
//!
//! ## Usage
//! Call `my_routes()` to get a configured `Router` for `/my` endpoints to be mounted in the main app.

use axum::{routing::get, Router};
use util::state::AppState;

pub mod announcements;
pub mod assignments;
pub mod tickets;
use tickets::get_my_tickets;

/// Builds and returns the `/my` route group.
///
/// Routes:
/// - `GET /my/announcements` → fetch announcements for the logged-in user
/// - `GET /my/tickets`       → fetch tickets for the logged-in user
/// - `GET /my/assignments`   → fetch assignments for the logged-in user
///
/// All routes operate on the currently authenticated user and require the application state.
pub fn my_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/announcements", get(announcements::get_my_announcements))
        .route("/tickets", get(get_my_tickets))
        .route("/assignments", get(assignments::get_my_assignments))
        .with_state(app_state)
}
