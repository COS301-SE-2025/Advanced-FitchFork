use axum::Router;
use util::state::AppState;
pub mod announcements;
pub mod tickets;
pub mod assignments;
pub fn my_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
}
