use axum::{routing::get, Router};
use util::state::AppState;
pub mod announcements;
pub mod assignments;
pub mod tickets;
use tickets::get_my_tickets;
pub fn my_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        //   .route("/announcements", axum::routing::get(announcements::get_my_announcements))
        .route("/tickets", get(get_my_tickets))
        // .route("/assignments", axum::routing::get(assignments::get_my_assignments))
        .with_state(app_state)
}
