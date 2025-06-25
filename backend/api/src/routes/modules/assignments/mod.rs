pub mod delete;
pub mod get;
pub mod post;
pub mod put;
mod config;
pub mod mark_allocator;
pub mod submissions;
pub mod memo_output;

use axum::{
    extract::Path,
    middleware::from_fn,
    routing::{delete, get, post, put},
    Router,
};

use delete::{delete_assignment, delete_files};
use get::{download_file, get_assignment, get_assignments, stats, list_files};
use post::{create, upload_files};
use put::edit_assignment;
use config::config_routes;
use mark_allocator::mark_allocator_routes;
use submissions::submission_routes;

use crate::{auth::guards::{
    require_assigned_to_module, require_lecturer, require_lecturer_or_admin
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
/// - `GET  /assignments/:assignment_id/stats`         → Assignment statistics (lecturer only)
/// - `POST /assignments/:assignment_id/tasks`         → Create a new task (lecturer/admin only)
/// - `GET  /assignments/:assignment_id/tasks`         → List tasks (lecturer/admin only)
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
        .nest(
            "/:assignment_id/config",
            config_routes().layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_lecturer_or_admin(Path(params), req, next)
            })),
        )
        .nest(
            "/:assignment_id/memo_output",
            memo_output::memo_output_routes().layer(from_fn(|Path((assignment_id,)): Path<(i64,)>, req, next| {
                require_lecturer_or_admin(Path((assignment_id,)), req, next)
            })),
        )
        
    // TODO: The following route is commented out:
    // .route(
    //     "/:assignment_id/submissions",
    //     post(submit_assignment).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
    //         require_assigned_to_module(Path(params), req, next)
    //     })),
    // )
        .nest("/:assignment_id/mark-allocator", mark_allocator_routes())
        .nest("/:assignment_id/submissions", submission_routes())
}
