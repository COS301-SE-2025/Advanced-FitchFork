//! # Me Routes Module
//!
//! Defines and wires up routes for user-specific endpoints, such as announcements, tickets, and assignments.
//!
//! ## Structure
//! - `announcements.rs` — GET handlers for fetching the user's announcements
//! - `assignments.rs` — GET handlers for fetching the user's assignments
//! - `tickets.rs` — GET handlers for fetching the user's tickets
//! - `grades.rs` — GET handlers for fetching the user's grades
//! - `submissions.rs` — GET handlers for fetching the user's submissions
//! - `events.rs` — GET handlers for fetching the user's events
//!
//! ## Usage
//! Call `me_routes()` to get a configured `Router` for `/me` endpoints to be mounted in the main app.

use axum::{Router, routing::get};
use util::state::AppState;

pub mod announcements;
pub mod assignments;
pub mod events;
pub mod grades;
pub mod submissions;
pub mod tickets;

/// Builds and returns the `/me` route group.
///
/// Routes:
/// - `GET /me/announcements` → fetch announcements for the logged-in user
/// - `GET /me/tickets`       → fetch tickets for the logged-in user
/// - `GET /me/assignments`   → fetch assignments for the logged-in user
/// - `GET /me/grades`        → fetch grades for the logged-in user
/// - `GET /me/submissions`   → fetch submissions for the logged-in user
/// - `GET /me/events`        → fetch events for the logged-in user
///
/// All routes operate on the currently authenticated user and require the application state.
pub fn me_routes() -> Router {
    Router::new()
        .route("/announcements", get(announcements::get_my_announcements))
        .route("/tickets", get(tickets::get_my_tickets))
        .route("/assignments", get(assignments::get_my_assignments))
        .route("/grades", get(grades::get_my_grades))
        .route("/submissions", get(submissions::get_my_submissions))
        .route("/events", get(events::get_my_events))
}
