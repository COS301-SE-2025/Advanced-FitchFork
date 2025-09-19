//! WebSocket topic routes under `/api/ws/modules/...`.
//!
//! Contains real-time module-related routes such as announcements,
//! and nests assignment-specific WebSocket topics as well.

use axum::{Router, middleware::from_fn_with_state, routing::get};
use util::state::AppState;
use util::ws::default_websocket_handler;

use crate::auth::guards::allow_student;
use crate::ws::modules::assignments::ws_assignment_routes;

pub mod assignments;

/// Builds the `/api/ws/modules` WebSocket router.
///
/// # Routes
/// - `/api/ws/modules/{module_id}/announcements` (guarded by `allow_student`)
/// - `/api/ws/modules/assignments/...`
///
/// # Guards
/// - `allow_student` ensures only users assigned to the module can subscribe to its announcement topic
pub fn ws_module_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/{module_id}/announcements",
            get(default_websocket_handler).route_layer(from_fn_with_state(
                app_state.clone(),
                allow_student,
            )),
        )
        .nest("/{module_id}/assignments", ws_assignment_routes())
}
