pub mod post;
pub mod put;
pub mod get;

use axum::{Router, routing::post, routing::get, routing::put};
use post::{set_assignment_config,};
use get::{get_assignment_config,};
use put::{update_assignment_config,};

pub fn config_routes() -> Router {
    Router::new()
        .route("/", post(set_assignment_config))
        .route("/", get(get_assignment_config))
        .route("/", put(update_assignment_config))
}