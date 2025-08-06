use axum::{Router, routing::{get}};
use get::{list_plagiarism_cases, get_graph};
use util::state::AppState;

pub mod get;

pub fn plagiarism_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_plagiarism_cases))
        .route("/graph", get(get_graph))
}