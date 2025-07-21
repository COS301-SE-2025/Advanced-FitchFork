use axum::{middleware::from_fn, middleware::from_fn_with_state, Router, routing::{get, post, put, delete}};
use crate::auth::guards::{require_admin, require_student};
use get::get_students;
use post::assign_students;
use put::edit_students;
use delete::remove_students;
use sea_orm::DatabaseConnection;

mod get;
mod post;
mod put;
mod delete;

/// Builds and returns the `/api/modules/{module_id}/students` route group.
///
/// - `GET` is accessible to students assigned to the module.
/// - `POST`, `PUT`, and `DELETE` are admin-only.
///
/// # Routes
/// - `GET    /`     → get students assigned to module
/// - `POST   /`     → assign students
/// - `PUT    /`     → set students (overwrites existing roles)
/// - `DELETE /`     → remove students from module
pub fn student_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/", get(get_students).route_layer(from_fn_with_state(db.clone(), require_student)))
        .route("/", post(assign_students).route_layer(from_fn(require_admin)))
        .route("/", put(edit_students).route_layer(from_fn(require_admin)))
        .route("/", delete(remove_students).route_layer(from_fn(require_admin)))
}