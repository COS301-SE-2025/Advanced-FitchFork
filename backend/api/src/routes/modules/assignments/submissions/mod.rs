pub mod get;
pub mod post;
pub mod common;

use axum::middleware::from_fn;
use axum::{extract::Path, Router};
use get::{list_submissions, get_submission};
use axum::routing::{get, post};
use post::{submit_assignment};

use crate::auth::guards::{require_assigned_to_module};

/// Defines HTTP routes related to assignment submissions.
///
/// # Routes
///
/// - `GET /`  
///   Returns a list of submissions for the assignment:
///   - **Lecturer/Tutor**: Returns all submissions for the assignment.
///   - **Student**: Returns only the student's own submission.
///
/// - `GET /{submission_id}`  
///   Returns a specific submission by ID:
///   - Access is restricted to users assigned to the module.
/// 
/// - `POST /`
///   - Submit a new assignment (student access only)
pub fn submission_routes() -> Router {
    Router::new()
        .route(
            "/",
            get(list_submissions).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_assigned_to_module(Path(params), req, next)
            })),
        )
          .route(
            "/{submission_id}",
            get(get_submission).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_assigned_to_module(Path(params), req, next)
            })),
        )
        .route(
            "/",
            post(submit_assignment).layer(from_fn(|Path(params): Path<(i64,)>, req, next| {
                require_assigned_to_module(Path(params), req, next)
            })),
        )
}