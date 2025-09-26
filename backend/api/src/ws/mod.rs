// api/src/ws/mod.rs
use axum::{Router, middleware::from_fn, routing::get};
use util::state::AppState;

use crate::auth::guards::allow_authenticated;

pub mod auth;
mod mux;
pub mod types;

pub mod attendance;
pub mod core;
pub mod submissions;
pub mod system;
pub mod tickets;

pub fn ws_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(mux::ws_multiplex_entry))
        .route_layer(from_fn(allow_authenticated))
        .with_state(app_state)
}
