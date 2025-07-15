pub mod get;
pub mod post;

use axum::{Router, routing::{post, get}};
use post::generate_memo_output;
use get::get_all_memo_outputs;

/// Handles memo output functionality for assignments.
/// Expects `module_id` and `assignment_id` path parameters at a higher level.
///
/// Routes:
/// - `POST /generate`      → Start async memo output generation for an assignment
/// - `GET  /`              → Retrieve all memo outputs for an assignment
pub fn memo_output_routes() -> Router {
    Router::new()
        .route(
            "/generate",
            post(generate_memo_output)
        )
        .route(
            "/",
            get(get_all_memo_outputs)
        )
}