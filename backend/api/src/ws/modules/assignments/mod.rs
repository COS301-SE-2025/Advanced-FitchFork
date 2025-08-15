//! WebSocket topic routes for assignment-related communication.
//!
//! Includes real-time submission progress updates and ticket chat channels.

use axum::{routing::get, Router};
use util::ws::default_websocket_handler;

/// Builds the `/api/ws/modules/assignments` WebSocket router.
///
/// # Routes
/// - `/api/ws/modules/assignments/{assignment_id}/submissions/{submission_id}/progress` // TODO: restrict to owner (student) or elevated roles
/// - `/api/ws/modules/assignments/{assignment_id}/tickets/{ticket_id}/chat`             // TODO: restrict to ticket owner (student) or elevated roles
pub fn ws_assignment_routes() -> Router {
    Router::new()
        .route("/{assignment_id}/submissions/{submission_id}/progress", get(default_websocket_handler)) // TODO: guard - student only for own submission
        .route("/{assignment_id}/tickets/{ticket_id}/chat", get(default_websocket_handler))             // TODO: guard - student only for own ticket
}
