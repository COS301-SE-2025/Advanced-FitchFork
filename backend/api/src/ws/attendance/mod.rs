use axum::{routing::get, middleware::from_fn_with_state, Router};
use util::state::AppState;

pub mod topics;
pub mod handlers;
pub mod ws_handlers;
pub mod common;

use crate::auth::guards::{allow_attendance_ws_access};
use handlers::attendance_session_ws_handler;

pub fn ws_attendance_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/sessions/{session_id}", get(attendance_session_ws_handler))
    .route_layer(from_fn_with_state(app_state.clone(), allow_attendance_ws_access))
}