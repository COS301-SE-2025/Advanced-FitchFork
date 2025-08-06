use axum::Router;
use util::state::AppState;

pub mod post;
pub mod get;
pub mod delete;
pub mod put;


pub fn announcement_routes(app_state: AppState) -> Router<AppState> {
    Router::new()

}