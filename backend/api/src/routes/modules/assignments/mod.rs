use axum::{middleware::from_fn_with_state, Router, routing::{get, post, put, delete}};
use config::config_routes;
use memo_output::memo_output_routes;
use delete::delete_assignment;
use get::{get_assignment, get_assignments, get_assignment_stats, get_assignment_readiness};
use mark_allocator::mark_allocator_routes;
use post::create_assignment;
use put::edit_assignment;
use submissions::submission_routes;
use files::files_routes;
use tasks::tasks_routes;
use crate::auth::guards::{require_assigned_to_module, require_lecturer};
use sea_orm::DatabaseConnection;

pub mod config;
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

/// Expects a module ID
/// If an assignment ID is included it will be deleted
/// - `POST /`                      → `create`
/// - `DELETE /{assignment_id}`      → `delete_assignment`
/// Builds and returns the `/assignments` route group.
///
/// Routes:
/// - `POST /assignments`                               → Create a new assignment
/// - `GET  /assignments`                               → List assignments
/// - `GET  /assignments/{assignment_id}`                → Get assignment details
/// - `PUT  /assignments/{assignment_id}`                → Edit assignment
/// - `DELETE /assignments/{assignment_id}`              → Delete assignment
/// - `GET  /assignments/{assignment_id}/stats`          → Assignment statistics (lecturer only)
/// - `GET  /assignments/{assignment_id}/readiness`      → Assignment readiness (lecturer or admin only)
///
/// Nested routes:
/// - Tasks routes              → `tasks_routes`
/// - Config routes             → `config_routes`
/// - Memo output routes        → `memo_output_routes`
/// - Mark allocator routes     → `mark_allocator_routes`
/// - Submissions routes        → `submission_routes`
/// - Files routes              → `files_routes`
pub fn assignment_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/", post(create_assignment).route_layer(from_fn_with_state(db.clone(), require_lecturer)))
        .route("/", get(get_assignments).route_layer(from_fn_with_state(db.clone(), require_assigned_to_module)))
        .route("/{assignment_id}", get(get_assignment).route_layer(from_fn_with_state(db.clone(), require_assigned_to_module)))
        .route("/{assignment_id}", put(edit_assignment).route_layer(from_fn_with_state(db.clone(), require_lecturer)))
        .route("/{assignment_id}", delete(delete_assignment).route_layer(from_fn_with_state(db.clone(), require_lecturer)))
        .route("/{assignment_id}/stats", get(get_assignment_stats).route_layer(from_fn_with_state(db.clone(), require_lecturer)))
        .route("/{assignment_id}/readiness", get(get_assignment_readiness).route_layer(from_fn_with_state(db.clone(), require_lecturer)))
        .nest("/{assignment_id}/tasks", tasks_routes().route_layer(from_fn_with_state(db.clone(), require_lecturer)))
        .nest("/{assignment_id}/config", config_routes().layer(from_fn_with_state(db.clone(), require_lecturer)))
        .nest("/{assignment_id}/memo_output", memo_output_routes().layer(from_fn_with_state(db.clone(), require_lecturer)))
        .nest("/{assignment_id}/mark_allocator", mark_allocator_routes().route_layer(from_fn_with_state(db.clone(), require_lecturer)))
        .nest( "/{assignment_id}/submissions", submission_routes().route_layer(from_fn_with_state(db.clone(), require_assigned_to_module)))
        .nest("/{assignment_id}/files", files_routes(db.clone()))
}