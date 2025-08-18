use axum::Router;
use util::state::AppState;




pub fn ws_submission_routes(_: AppState) -> Router<AppState> {
    Router::new()
}