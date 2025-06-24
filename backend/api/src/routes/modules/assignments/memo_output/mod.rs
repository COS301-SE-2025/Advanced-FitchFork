pub mod get;
pub mod post;

use axum::{Router, routing::post};

use post::generate_memo_output;

pub fn memo_output_routes() -> Router {
    Router::new()
        .route("/generate", post(generate_memo_output))
}