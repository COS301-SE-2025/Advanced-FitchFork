mod config;
pub mod delete;
pub mod get;
pub mod post;
pub mod put;
pub mod mark_allocator;
pub mod submissions;
pub mod files;
pub mod memo_output;
pub mod tasks;
pub mod common;

use axum::{
    extract::Path,
    middleware::from_fn,
    routing::{delete, get, post, put},
    Router,
};

use config::config_routes;
use memo_output::memo_output_routes;
use delete::delete_assignment;
use get::{get_assignment, get_assignments, stats};
use mark_allocator::mark_allocator_routes;
use post::create;
use put::edit_assignment;
use submissions::submission_routes;
use files::files_routes;
use tasks::tasks_routes;

use crate::{
    auth::guards::{require_lecturer, require_lecturer_or_admin},
};

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
/// - `DELETE /assignments/:assignment_id`        → Delete assignment
/// - `GET  /assignments/:assignment_id/stats`         → Assignment statistics (lecturer only)
pub fn assignment_routes() -> Router {
    Router::new()
        .route(
            "/",
            post(create)
        )
        .route(
            "/",
            get(get_assignments)
        )
        .route(
            "/:assignment_id",
            get(get_assignment)
        )
        .route(
            "/:assignment_id",
            put(edit_assignment)
        )
        .route(
            "/:assignment_id",
            delete(delete_assignment)
        )
        .route(
            "/:assignment_id/stats",
            get(stats).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_lecturer(Path(params), req, next)
            })),
        )
        .nest(
            "/:assignment_id/tasks",
            tasks_routes()
        )
        .nest(
            "/:assignment_id/config",
            config_routes().layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_lecturer_or_admin(Path(params), req, next)
            })),
        )
        .nest(
            "/:assignment_id/memo_output",
            memo_output_routes().layer(from_fn(|Path((assignment_id,)): Path<(i64,)>, req, next| {
                require_lecturer_or_admin(Path((assignment_id,)), req, next)
            })),
        )
        .nest(
            "/:assignment_id/mark_allocator",
            mark_allocator_routes()
        )
        .nest(
            "/:assignment_id/submissions",
            submission_routes()
        )
        .nest(
            "/:assignment_id/files",
            files_routes()
        )
}