use axum::{middleware::from_fn, Router};
use util::state::AppState;

use crate::{
    auth::guards::require_authenticated,
    ws::{modules::ws_module_routes, tickets::ws_ticket_routes},
};

pub mod modules;
pub mod tickets;

pub fn ws_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/modules", ws_module_routes(app_state.clone()))
        .nest("/tickets", ws_ticket_routes(app_state.clone()))
        .route_layer(from_fn(require_authenticated))
        .with_state(app_state)
}