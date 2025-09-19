use axum::{middleware::{from_fn_with_state}, routing::get, Router};
use util::state::AppState;

use crate::{auth::guards::allow_ticket_ws_access, ws::tickets::handlers::ticket_chat_handler};

pub mod topics;
pub mod handlers;
pub mod ws_handlers;
pub mod common;

pub fn ws_ticket_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/{ticket_id}", get(ticket_chat_handler))
    .route_layer(from_fn_with_state(app_state.clone(), allow_ticket_ws_access))
}