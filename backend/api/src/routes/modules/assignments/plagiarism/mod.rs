use axum::{routing::{delete, get, patch, post, put}, Router};
use get::{list_plagiarism_cases, get_graph};
use post::create_plagiarism_case;
use put::update_plagiarism_case;
use delete::{delete_plagiarism_case, bulk_delete_plagiarism_cases};
use patch::{patch_plagiarism_flag, patch_plagiarism_review};
use util::state::AppState;

pub mod get;
pub mod post;
pub mod put;
pub mod delete;
pub mod patch;

/// Builds and returns the `/assignments/plagiarism` route group.
///
/// Routes:
/// - `GET    /assignments/plagiarism`                          → List plagiarism cases
/// - `GET    /assignments/plagiarism/graph`                    → Get plagiarism graph
/// - `POST   /assignments/plagiarism`                          → Create a new plagiarism case
/// - `PUT    /assignments/plagiarism/{plagiarism_id}`          → Update a plagiarism case
/// - `DELETE /assignments/plagiarism/{plagiarism_id}`          → Delete a plagiarism case
/// - `DELETE /assignments/plagiarism/bulk`                     → Bulk delete plagiarism cases
/// - `PATCH  /assignments/plagiarism/{plagiarism_id}/flag`     → Flag a plagiarism case
/// - `PATCH  /assignments/plagiarism/{plagiarism_id}/review`   → Review a plagiarism case
pub fn plagiarism_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_plagiarism_cases))
        .route("/graph", get(get_graph))
        .route("/", post(create_plagiarism_case))
        .route("/{plagiarism_id}", put(update_plagiarism_case))
        .route("/{plagiarism_id}", delete(delete_plagiarism_case))
        .route("/bulk", delete(bulk_delete_plagiarism_cases))
        .route("/{plagiarism_id}/flag", patch(patch_plagiarism_flag))
        .route("/{plagiarism_id}/review", patch(patch_plagiarism_review))
}