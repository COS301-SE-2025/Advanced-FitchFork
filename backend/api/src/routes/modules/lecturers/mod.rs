use axum::{middleware::from_fn, Router, routing::{get, post, put, delete}};
use crate::auth::guards::{require_admin, require_lecturer};
use get::get_lecturers;
use post::assign_lecturers;
use put::edit_lecturers;
use delete::remove_lecturers;

mod get;
mod post;
mod put;
mod delete;

/// Builds and returns the `/api/modules/{module_id}/lecturers` route group.
///
/// - `GET` is accessible to lecturers assigned to the module.
/// - `POST`, `PUT`, and `DELETE` are admin-only.
///
/// # Routes
/// - `GET    /modules/{module_id}/lecturers`     → get lecturers assigned to module
/// - `POST   /modules/{module_id}/lecturers`     → assign lecturers
/// - `PUT    /modules/{module_id}/lecturers`     → set lecturers (overwrites existing roles)
/// - `DELETE /modules/{module_id}/lecturers`     → remove lecturers from module
pub fn lecturer_routes() -> Router {
    Router::new()
        .route("/", get(get_lecturers).route_layer(from_fn(require_lecturer)))
        .route("/", post(assign_lecturers).route_layer(from_fn(require_admin)))
        .route("/", put(edit_lecturers).route_layer(from_fn(require_admin)))
        .route("/", delete(remove_lecturers).route_layer(from_fn(require_admin)))
}