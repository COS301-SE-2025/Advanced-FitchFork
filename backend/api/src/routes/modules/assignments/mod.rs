use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub mod delete;
pub mod get;
pub mod post;
pub mod put;

use delete::delete_assignment;

use get::{get_assignment, get_assignments,download_file};
use post::{create, upload_files};
use put::edit_assignment;
use crate::routes::modules::assignments::get::list_files;

/// Expects a module ID
/// If an assignment ID is included it will be deleted
/// - `POST /` → `create` 
/// - `DELETE /:assignment_id` → `delete_assignment`
/// Builds and returns the `/assignments` route group.
///
/// Routes:
/// - `POST /assignments`               → Create a new assignment
/// - `GET  /assignments`               → List assignments (with optional filters)
/// - `GET  /assignments/:assignment_id` → Get details of a specific assignment
/// - `PUT  /assignments/:assignment_id` → Edit an existing assignment
/// - `DELETE /assignments/:assignment_id` → Delete an assignment
/// - `POST /assignments/:assignment_id/files` → Upload files for an assignment
///
/// Note: Expects a module ID to be part of the parent route, i.e., nested under `/modules/:module_id/assignments`.
pub fn assignment_routes() -> Router {
    Router::new()
        .route("/", post(create))
        .route("/", get(get_assignments))
        .route("/:assignment_id", get(get_assignment))
        .route("/:assignment_id", put(edit_assignment))
        .route("/:assignment_id/files", post(upload_files))
        .route("/:assignment_id/file/:file_id", get(download_file))
        .route("/:assignment_id/files", get(list_files))
        .route("/:assignment_id", delete(delete_assignment))
}
