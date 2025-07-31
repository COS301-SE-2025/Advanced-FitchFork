use axum::{Router, routing::get};
pub mod get;
use get::get_graph;
use sea_orm::DatabaseConnection;

pub fn plagiarism_routes() -> Router<DatabaseConnection> {
    Router::new().route("/", get(get_graph))
}
