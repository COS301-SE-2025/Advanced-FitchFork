//! Task Routes Module
//!
//! This module defines the routing for assignment task-related endpoints, including retrieving and editing task details. It applies access control middleware to ensure only lecturers or admins can access these endpoints.

pub mod get;
pub mod put;

use axum::{
    extract::Path,
    middleware::from_fn,
    routing::{get, put},
    Router,
};

use crate::auth::guards::require_lecturer_or_admin;
use get::get_task_details;
use put::edit_task;

/// Registers the routes for assignment task endpoints.
///
/// This function sets up the following endpoints under the current router:
///
/// - `GET /:task_id`: Retrieves detailed information about a specific task. Access is restricted to users with lecturer or admin roles for the assignment.
/// - `PUT /:task_id`: Edits the command of a specific task. Access is restricted to users with lecturer or admin roles for the assignment.
///
/// Both routes apply the `require_lecturer_or_admin` middleware, which checks the user's role for the assignment before allowing access.
///
/// # Returns
/// An [`axum::Router`] with the task endpoints and their associated middleware.
pub fn tasks_routes() -> Router {
    Router::new()
        .route(
            "/:task_id",
            get(get_task_details).layer(from_fn(|Path((assignment_id, _task_id)): Path<(i64, i64)>, req, next| {
                require_lecturer_or_admin(Path((assignment_id,)), req, next)
            })),
        )
        .route(
            "/:task_id",
            put(edit_task).layer(from_fn(|Path((assignment_id, _task_id)): Path<(i64, i64)>, req, next| {
                require_lecturer_or_admin(Path((assignment_id,)), req, next)
            })),
        )
}