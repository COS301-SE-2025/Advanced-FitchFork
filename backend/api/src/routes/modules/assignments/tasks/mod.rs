//! Task Routes Module
//!
//! This module defines the routing for assignment task-related endpoints, including retrieving, editing, creating, and deleting task details. It applies access control middleware to ensure only lecturers or admins can access these endpoints.

pub mod get;
pub mod put;
pub mod post;
pub mod delete;
pub mod common;

use axum::{
    extract::Path,
    middleware::from_fn,
    routing::{get, put, post, delete},
    Router,
};

use crate::auth::guards::require_lecturer_or_admin;
use get::{get_task_details, list_tasks};
use put::edit_task;
use post::create_task;
use delete::delete_task;

/// Registers the routes for assignment task endpoints.
///
/// This function sets up the following endpoints under the current router:
///
/// - `GET /`: Lists all tasks for the assignment. Access is restricted to users with lecturer or admin roles for the assignment.
/// - `POST /`: Creates a new task for the assignment. Access is restricted to users with lecturer or admin roles for the assignment.
/// - `GET /{task_id}`: Retrieves detailed information about a specific task. Access is restricted to users with lecturer or admin roles for the assignment.
/// - `PUT /{task_id}`: Edits the command of a specific task. Access is restricted to users with lecturer or admin roles for the assignment.
/// - `DELETE /{task_id}`: Deletes a specific task from the assignment. Access is restricted to users with lecturer or admin roles for the assignment.
///
/// All routes apply the `require_lecturer_or_admin` middleware, which checks the user's role for the assignment before allowing access.
///
/// # Returns
/// An [`axum::Router`] with the task endpoints and their associated middleware.
pub fn tasks_routes() -> Router {
    Router::new()
        .route(
            "/",
            get(list_tasks).layer(from_fn(|Path((assignment_id,)): Path<(i64,)>, req, next| {
                require_lecturer_or_admin(Path((assignment_id,)), req, next)
            })),
        )
        .route(
            "/",
            post(create_task).layer(from_fn(|Path((assignment_id,)): Path<(i64,)>, req, next| {
                require_lecturer_or_admin(Path((assignment_id,)), req, next)
            })),
        )
        .route(
            "/{task_id}",
            get(get_task_details).layer(from_fn(|Path((assignment_id, _task_id)): Path<(i64, i64)>, req, next| {
                require_lecturer_or_admin(Path((assignment_id,)), req, next)
            })),
        )
        .route(
            "/{task_id}",
            put(edit_task).layer(from_fn(|Path((assignment_id, _task_id)): Path<(i64, i64)>, req, next| {
                require_lecturer_or_admin(Path((assignment_id,)), req, next)
            })),
        )
        .route(
            "/{task_id}",
            delete(delete_task).layer(from_fn(|Path((assignment_id, _task_id)): Path<(i64, i64)>, req, next| {
                require_lecturer_or_admin(Path((assignment_id,)), req, next)
            })),
        )
}