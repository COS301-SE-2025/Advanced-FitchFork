use axum::{middleware::from_fn, Router};
use util::state::AppState;

use crate::{
    auth::guards::allow_authenticated,
    ws::{
        modules::ws_module_routes,
        tickets::ws_ticket_routes,
        attendance::ws_attendance_routes,
    },
};

pub mod modules;
pub mod tickets;
pub mod attendance;

pub fn ws_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/modules", ws_module_routes(app_state.clone()))
        .nest("/tickets", ws_ticket_routes(app_state.clone()))
        .nest("/attendance", ws_attendance_routes(app_state.clone())) // ← ADD
    .route_layer(from_fn(allow_authenticated))
        .with_state(app_state)
}
