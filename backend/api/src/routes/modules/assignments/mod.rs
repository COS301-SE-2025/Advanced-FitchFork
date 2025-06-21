pub mod delete;
pub mod get;
pub mod post;
pub mod put;
mod submissions;

use axum::{
    extract::Path,
    middleware::from_fn,
    routing::{delete, get, post, put},
    Router,
};

use delete::{delete_assignment, delete_files};
use get::{download_file, get_assignment, get_assignments, get_my_submissions, list_submissions, stats, list_files};
use post::{create, upload_files};
use put::edit_assignment;

use crate::{auth::guards::{
    require_admin, require_assigned_to_module, require_lecturer, require_lecturer_or_admin, require_lecturer_or_tutor
}, routes::modules::assignments::{get::list_tasks, post::create_task}};

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
/// - `GET  /assignments/:assignment_id/submissions` → List all submissions (lecturer/tutor only)
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
            "/:assignment_id/submissions/me",
            get(get_my_submissions).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_assigned_to_module(Path(params), req, next)
            })),
        )
        .route(
            "/:assignment_id/submissions",
            get(list_submissions).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_lecturer_or_tutor(Path(params), req, next)
            })),
        )
        .route(
            "/:assignment_id/stats",
            get(stats).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_lecturer(Path(params), req, next)
            })),
        ).route(
            "/:assignment_id/tasks",
            post(create_task).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_lecturer_or_admin(Path(params), req, next)
            })),
        )
        .route(
            "/:assignment_id/tasks",
            get(list_tasks).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_lecturer_or_admin(Path(params), req, next)
            })),
        )
        .route("/:assignment_id", delete(delete_assignment))
    // TODO: The following route is commented out:
    // .route(
    //     "/:assignment_id/submissions",
    //     post(submit_assignment).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
    //         require_assigned_to_module(Path(params), req, next)
    //     })),
    // )
}
