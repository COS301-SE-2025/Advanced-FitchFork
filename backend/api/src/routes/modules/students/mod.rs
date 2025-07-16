mod get;
mod post;
mod put;
mod delete;

use axum::{
    middleware::from_fn,
    routing::{get, post, put, delete},
    extract::Path,
    Router,
};
use crate::auth::guards::{require_admin, require_student};
use get::get_students;
use post::assign_students;
use put::edit_students;
use delete::remove_students;

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
pub fn student_routes() -> Router {
    Router::new()
        .route(
            "/",
            get(get_students).route_layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_student(Path(params), req, next)
            })),
        )
        .route(
            "/",
            post(assign_students).route_layer(from_fn(require_admin))
        )
        .route(
            "/",
            put(edit_students).route_layer(from_fn(require_admin))
        )
        .route(
            "/", 
            delete(remove_students).route_layer(from_fn(require_admin))
        )
}