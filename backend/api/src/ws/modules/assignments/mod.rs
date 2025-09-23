// api/src/ws/modules/assignments/mod.rs
use axum::Router;
use util::state::AppState;

pub mod submissions;

pub fn ws_assignment_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .nest(
            "/{assignment_id}/submissions",
            submissions::ws_assignment_submission_routes(app_state.clone()),
        )
        .with_state(app_state)
}
