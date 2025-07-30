/// Routes under `/assignments/{assignment_id}/config`:
///
/// - `POST /` → Save or replace the assignment configuration on disk.  
///   Accepts a JSON object matching the [`ExecutionConfig`] schema (see `/default` for structure).
///
/// - `GET /` → Load the assignment configuration from disk if it exists; otherwise returns an empty object.  
///   The format is based on the [`ExecutionConfig`] struct from `util::execution_config`.
///
/// - `GET /default` → Returns the system's default [`ExecutionConfig`] used to initialize new configurations.
///
/// Configuration files are stored under:
/// `ASSIGNMENT_STORAGE_ROOT/module_{id}/assignment_{id}/config/config.json`
use axum::{
    Router,
    routing::{get, post},
};
use get::{get_assignment_config, get_default_assignment_config};
use post::set_assignment_config;
use sea_orm::DatabaseConnection;

pub mod get;
pub mod post;

pub fn config_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/", post(set_assignment_config))
        .route("/", get(get_assignment_config))
        .route("/default", get(get_default_assignment_config))
}
