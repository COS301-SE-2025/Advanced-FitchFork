/// Routes under `/assignments/{assignment_id}/config`:
///
/// - `POST /` → Set or replace the full assignment configuration (must be a JSON object)
/// - `GET /`  → Retrieve the current assignment configuration (returns object or empty)
/// - `PUT /`  → Partially update known fields of the existing configuration

use axum::{Router, routing::{get, post, put}};
use post::set_assignment_config;
use get::get_assignment_config;
use put::update_assignment_config;
use sea_orm::DatabaseConnection;

pub mod post;
pub mod put;
pub mod get;

pub fn config_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .with_state(db.clone())
        .route("/", post(set_assignment_config))
        .route("/", get(get_assignment_config))
        .route("/", put(update_assignment_config))
}