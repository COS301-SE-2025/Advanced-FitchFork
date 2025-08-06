use axum::{Router, routing::{get, post, put, delete}};
use get::{list_plagiarism_cases, get_graph};
use post::create_plagiarism_case;
use put::update_plagiarism_case;
use delete::delete_plagiarism_case;
use util::state::AppState;

pub mod get;
pub mod post;
pub mod put;
pub mod delete;

/// Builds and returns the `/assignments/plagiarism` route group.
///
/// Routes:
/// - `GET    /assignments/plagiarism`                      → List plagiarism cases
/// - `POST   /assignments/plagiarism`                      → Create a new plagiarism case
/// - `PUT    /assignments/plagiarism/:submission_id`       → Update a plagiarism case
/// - `DELETE /assignments/plagiarism/:submission_id`       → Delete a plagiarism case
/// - `GET    /assignments/plagiarism/graph`                → Get plagiarism graph
pub fn plagiarism_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_plagiarism_cases))
        .route("/", post(create_plagiarism_case))
        .route("/:submission_id", put(update_plagiarism_case))
        .route("/:submission_id", delete(delete_plagiarism_case))
        .route("/graph", get(get_graph))
}