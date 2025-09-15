use axum::{middleware::from_fn, routing::get, Router};

use crate::{auth::guards::require_ticket_ws_access, ws::tickets::handlers::ticket_chat_handler};

pub mod topics;
pub mod handlers;
pub mod ws_handlers;
pub mod common;

pub fn ws_ticket_routes() -> Router {
    Router::new()
        .route("/{ticket_id}", get(ticket_chat_handler))
        .route_layer(from_fn(require_ticket_ws_access))
}