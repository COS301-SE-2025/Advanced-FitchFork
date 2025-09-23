// mod.rs
use axum::{Router, middleware::from_fn_with_state, routing::get};
use util::state::AppState;

use crate::auth::guards::{
    allow_assistant_lecturer,
    allow_submission_ws_owner_only,
};

use crate::ws::modules::assignments::submissions::handlers::{
    submission_staff_ws_handler,
    submission_ws_handler,
};

pub mod common;
pub mod handlers;
pub mod topics;
pub mod ws_handlers;

pub fn ws_assignment_submission_routes(app_state: AppState) -> Router<AppState> {
    // /{user_id} → owner-only
    let owner_only = Router::new()
        .route("/{user_id}", get(submission_ws_handler))
        .route_layer(from_fn_with_state(
            app_state.clone(),
            allow_submission_ws_owner_only,
        ));

    // / (no user) → staff-only (Lecturer / AssistantLecturer)
    let staff_only = Router::new()
        .route("/staff", get(submission_staff_ws_handler))
        .route_layer(from_fn_with_state(
            app_state.clone(),
            allow_assistant_lecturer,
        ));

    owner_only.merge(staff_only).with_state(app_state)
}
