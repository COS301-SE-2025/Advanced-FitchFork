//! WebSocket route entry point for `/api/ws/...`.
//!
//! This module defines all WebSocket entry points under the `/api/ws` namespace.
//! WebSocket topics are organized by domain (e.g., modules, assignments), 
//! each protected via appropriate access control middleware.

use axum::{
    middleware::from_fn, routing::get, Router
};
use crate::{auth::guards::require_authenticated, ws::{handlers::chat_handler, modules::ws_module_routes}};

pub mod modules;
pub mod handlers;

/// Builds the `/ws` router containing all WebSocket topic namespaces.
///
/// # Routes
/// - `/ws/modules/...` → module-related real-time endpoints
///
/// # Middleware
/// - Applies `require_authenticated` globally to all WebSocket routes.
///
/// # Example
/// ```text
/// /ws/modules/{module_id}/announcements
/// /ws/modules/assignments/{assignment_id}/submissions/{submission_id}/progress
/// ```
pub fn ws_routes() -> Router {
    Router::new()
        .route("/chat", get(chat_handler).route_layer(from_fn(require_authenticated)))
        .nest("/modules", ws_module_routes())
}
