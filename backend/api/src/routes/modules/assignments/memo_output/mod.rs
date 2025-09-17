use axum::{
    Router,
    routing::{get, post},
};
use get::get_all_memo_outputs;
use post::generate_memo_output;
use util::state::AppState;

pub mod get;
pub mod post;

/// Handles memo output functionality for assignments.
/// Expects `module_id` and `assignment_id` path parameters at a higher level.
///
/// Routes:
/// - `POST /generate`      → Start async memo output generation for an assignment
/// - `GET  /`              → Retrieve all memo outputs for an assignment
pub fn memo_output_routes() -> Router<AppState> {
    Router::new()
        .route("/generate", post(generate_memo_output))
        .route("/", get(get_all_memo_outputs))
}
