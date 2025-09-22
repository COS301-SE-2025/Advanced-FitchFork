use axum::{Router, middleware::from_fn_with_state, routing::get};
use util::state::AppState;

use crate::{auth::guards::allow_submission_ws_owner_only, ws::modules::assignments::submissions::handlers::submission_ws_handler};

pub mod topics;
pub mod common;
pub mod ws_handlers;
pub mod handlers;

pub fn ws_assignment_submission_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/{user_id}", get(submission_ws_handler))
        .route_layer(from_fn_with_state(app_state.clone(), allow_submission_ws_owner_only))
        .with_state(app_state)
}
