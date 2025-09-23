use axum::{Router, middleware::from_fn_with_state, routing::get};
use util::state::AppState;

pub mod common;
pub mod handlers;
pub mod topics;
pub mod ws_handlers;

use crate::auth::guards::allow_attendance_ws_access;
use handlers::attendance_session_ws_handler;

pub fn ws_attendance_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/sessions/{session_id}", get(attendance_session_ws_handler))
        .route_layer(from_fn_with_state(
            app_state.clone(),
            allow_attendance_ws_access,
        ))
}
