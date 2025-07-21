use axum::{middleware::from_fn, middleware::from_fn_with_state, Router, routing::{get, post, put, delete}};
use crate::auth::guards::{require_admin, require_tutor};
use get::get_tutors;
use post::assign_tutors;
use put::edit_tutors;
use delete::remove_tutors;
use sea_orm::DatabaseConnection;

mod get;
mod post;
mod put;
mod delete;

/// Builds and returns the `/api/modules/{module_id}/tutors` route group.
///
/// - `GET` is accessible to tutors assigned to the module.
/// - `POST`, `PUT`, and `DELETE` are admin-only.
///
/// # Routes
/// - `GET    /`     → get tutors assigned to module
/// - `POST   /`     → assign tutors
/// - `PUT    /`     → set tutors (overwrites existing roles)
/// - `DELETE /`     → remove tutors from module
pub fn tutor_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/", get(get_tutors).route_layer(from_fn_with_state(db.clone(), require_tutor)))
        .route("/", post(assign_tutors).route_layer(from_fn(require_admin)))
        .route("/", put(edit_tutors).route_layer(from_fn(require_admin)))
        .route("/", delete(remove_tutors).route_layer(from_fn(require_admin)))
}