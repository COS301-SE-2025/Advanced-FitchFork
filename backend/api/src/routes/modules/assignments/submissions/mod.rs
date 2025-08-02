use axum::{middleware::from_fn_with_state, routing::{get, post}, Router};
use get::{get_submission, list_submissions, get_submission_output};
use post::{submit_assignment, remark_submissions};
use crate::auth::guards::{require_lecturer_or_tutor, require_lecturer_or_assistant_lecturer};
use sea_orm::DatabaseConnection;

pub mod common;
pub mod get;
pub mod post;

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
pub fn submission_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    Router::new()
        .route("/", get(list_submissions))
        .route("/{submission_id}", get(get_submission))
        .route("/{submission_id}/output", get(get_submission_output).route_layer(from_fn_with_state(db.clone(), require_lecturer_or_tutor)))
        .route("/", post(submit_assignment))
        .route("/remark", post(remark_submissions).route_layer(from_fn_with_state(db.clone(), require_lecturer_or_assistant_lecturer)))
}
