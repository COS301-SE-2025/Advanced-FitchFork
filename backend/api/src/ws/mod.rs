use axum::{middleware::from_fn, Router};

use crate::{
    auth::guards::require_authenticated,
    ws::{modules::ws_module_routes, tickets::ws_ticket_routes},
};

pub mod modules;
pub mod tickets;

pub fn ws_routes() -> Router {
    Router::new()
        .nest("/modules", ws_module_routes())
        .nest("/tickets", ws_ticket_routes())
        .route_layer(from_fn(require_authenticated))
}