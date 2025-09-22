// ws_system.rs
use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::get,
};
use util::state::AppState;

pub mod handlers;
pub mod topics;

use crate::auth::guards::{allow_admin, allow_authenticated};
use handlers::{system_health_admin_ws_handler, system_health_ws_handler};

pub fn ws_system_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/health",
            get(system_health_ws_handler).route_layer(from_fn(allow_authenticated)),
        )
        .route(
            "/health/admin",
            get(system_health_admin_ws_handler)
                .route_layer(from_fn_with_state(app_state.clone(), allow_admin)),
        )
}
