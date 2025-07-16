pub mod post;
pub mod put;
pub mod get;

/// Routes under `/assignments/{assignment_id}/config`:
///
/// - `POST /` → Set or replace the full assignment configuration (must be a JSON object)
/// - `GET /`  → Retrieve the current assignment configuration (returns object or empty)
/// - `PUT /`  → Partially update known fields of the existing configuration

use axum::{Router, routing::post, routing::get, routing::put};
use post::{set_assignment_config,};
use get::{get_assignment_config,};
use put::{update_assignment_config,};

pub fn config_routes() -> Router {
    Router::new()
        .route(
            "/",
            post(set_assignment_config)
        )
        .route(
            "/",
            get(get_assignment_config)
        )
        .route(
            "/",
            put(update_assignment_config)
        )
}