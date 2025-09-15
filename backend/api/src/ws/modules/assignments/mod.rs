//! WebSocket topic routes for assignment-related communication.
//!
//! Includes real-time submission progress updates and per-ticket chat channels.
//!
//! Effective paths (this module is nested under `/ws/modules/{module_id}/assignments`):
//! - `/ws/modules/{module_id}/assignments/{assignment_id}/submissions/{submission_id}/progress`
//!
//! Both endpoints are WebSocket upgrade routes.

use axum::{routing::get, Router};
use util::ws::default_websocket_handler;

pub mod submissions;

/// Builds the `/ws/modules/{module_id}/assignments` WebSocket router.
pub fn ws_assignment_routes() -> Router {
    Router::new()
        .route("/{assignment_id}/submissions/{submission_id}/progress",get(default_websocket_handler))
}
