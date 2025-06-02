pub mod delete;
pub mod get;
pub mod post;
pub mod put;

use axum::{
    Router,
    extract::Path,
    routing::{get, post, put, delete},
    middleware::from_fn,
};

use delete::{delete_assignment, delete_files};
use get::{download_file, get_assignment, get_assignments, get_my_submissions};
use post::{create, upload_files, submit_assignment};
use put::edit_assignment;

use crate::auth::guards::{require_assigned_to_module, require_lecturer};
use crate::routes::modules::assignments::get::list_files;

/// Expects a module ID
/// If an assignment ID is included it will be deleted
/// - `POST /` → `create`
/// - `DELETE /:assignment_id` → `delete_assignment`
/// Builds and returns the `/assignments` route group.
///
/// Routes:
/// - `POST /assignments`                         → Create a new assignment
/// - `GET  /assignments`                         → List assignments
/// - `GET  /assignments/:assignment_id`          → Get assignment details
/// - `PUT  /assignments/:assignment_id`          → Edit assignment
/// - `POST /assignments/:assignment_id/files`    → Upload files
/// - `GET  /assignments/:assignment_id/files`    → List files
/// - `GET  /assignments/:assignment_id/file/:file_id` → Download a file
/// - `DELETE /assignments/:assignment_id/files`  → Delete files
/// - `DELETE /assignments/:assignment_id`        → Delete assignment
/// - `POST /assignments/:assignment_id/submissions` → Submit assignment
/// - `GET  /assignments/:assignment_id/submissions/me` → Get my submissions
pub fn assignment_routes() -> Router {
    Router::new()
        .route("/", post(create))
        .route("/", get(get_assignments))
        .route("/:assignment_id", get(get_assignment))
        .route("/:assignment_id", put(edit_assignment))
        .route(
            "/:assignment_id/files",
            post(upload_files).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_lecturer(Path(params), req, next)
            })),
        )
        .route(
            "/:assignment_id/files",
            get(list_files).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_assigned_to_module(Path(params), req, next)
            })),
        )
        .route(
            "/:assignment_id/files",
            delete(delete_files).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_lecturer(Path(params), req, next)
            })),
        )
        .route(
            "/:assignment_id/file/:file_id",
            get(download_file).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_assigned_to_module(Path(params), req, next)
            })),
        )
        .route(
            "/:assignment_id/submissions",
            post(submit_assignment).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_assigned_to_module(Path(params), req, next)
            })),
        )
        .route(
            "/:assignment_id/submissions/me",
            get(get_my_submissions).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_assigned_to_module(Path(params), req, next)
            })),
        )
        .route("/:assignment_id", delete(delete_assignment))
}

