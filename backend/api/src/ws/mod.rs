use axum::{Router, middleware::from_fn};

use crate::{
    auth::guards::require_authenticated,
    ws::{attendance::ws_attendance_routes, modules::ws_module_routes, tickets::ws_ticket_routes},
};

pub mod attendance;
pub mod modules;
pub mod tickets;

pub fn ws_routes() -> Router {
    Router::new()
        .nest("/modules", ws_module_routes(app_state.clone()))
        .nest("/tickets", ws_ticket_routes(app_state.clone()))
        .nest("/attendance", ws_attendance_routes(app_state.clone())) // ← ADD
        .route_layer(from_fn(require_authenticated))
        .with_state(app_state)
}
