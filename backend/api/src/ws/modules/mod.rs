//! WebSocket topic routes under `/api/ws/modules/...`.
//!
//! Contains real-time module-related routes such as announcements,
//! and nests assignment-specific WebSocket topics as well.

use axum::{middleware::from_fn_with_state, routing::get, Router};
use util::state::AppState;
use util::ws::default_websocket_handler;

use crate::ws::modules::assignments::ws_assignment_routes;
use crate::auth::guards::require_assigned_to_module;

pub mod assignments;

/// Builds the `/api/ws/modules` WebSocket router.
///
/// # Routes
/// - `/api/ws/modules/{module_id}/announcements` (guarded by `require_assigned_to_module`)
/// - `/api/ws/modules/assignments/...`
///
/// # Guards
/// - `require_assigned_to_module` ensures only users assigned to the module can subscribe to its announcement topic
pub fn ws_module_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/{module_id}/announcements", get(default_websocket_handler).route_layer(from_fn_with_state(app_state.clone(), require_assigned_to_module)))
        .nest("/{module_id}/assignments", ws_assignment_routes(app_state))
}
