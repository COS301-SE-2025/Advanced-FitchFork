use axum::{Router, routing::get};
pub mod get;
use get::get_graph;
use util::state::AppState;

pub fn plagiarism_routes() -> Router<AppState> {
    Router::new().route("/", get(get_graph))
}
