use axum::{Router, middleware::from_fn};
use util::state::AppState;

use crate::{
    auth::guards::allow_authenticated,
    ws::{
        attendance::ws_attendance_routes, modules::ws_module_routes, system::ws_system_routes,
        tickets::ws_ticket_routes,
    },
};

pub mod attendance;
pub mod modules;
pub mod system;
pub mod tickets;

pub fn ws_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/modules", ws_module_routes(app_state.clone()))
        .nest("/tickets", ws_ticket_routes(app_state.clone()))
        .nest("/attendance", ws_attendance_routes(app_state.clone()))
        .nest("/system", ws_system_routes(app_state.clone()))
        .route_layer(from_fn(allow_authenticated))
        .with_state(app_state)
}
