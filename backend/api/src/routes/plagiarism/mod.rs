use axum::{Router, routing::get};
pub mod get;
use get::get_graph;

pub fn plagiarism_routes() -> Router {
    Router::new().route("/", get(get_graph))
}