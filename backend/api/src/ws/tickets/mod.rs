use axum::{Router, middleware::from_fn, routing::get};

use crate::{auth::guards::require_ticket_ws_access, ws::tickets::handlers::ticket_chat_handler};

pub mod common;
pub mod handlers;
pub mod topics;
pub mod ws_handlers;

pub fn ws_ticket_routes() -> Router {
    Router::new()
        .route("/{ticket_id}", get(ticket_chat_handler))
        .route_layer(from_fn(require_ticket_ws_access))
}
