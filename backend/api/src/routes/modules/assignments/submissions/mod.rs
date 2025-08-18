use axum::{middleware::from_fn_with_state, routing::{get, post}, Router};
use get::{get_submission, list_submissions, get_submission_output};
use post::{submit_assignment, remark_submissions, resubmit_submissions};
use util::state::AppState;

use crate::auth::guards::{require_lecturer_or_assistant_lecturer, require_lecturer_or_tutor};

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
pub fn submission_routes(app_state : AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_submissions))
        .route("/{submission_id}", get(get_submission))
        .route("/{submission_id}/output", get(get_submission_output).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_tutor)))
        .route("/", post(submit_assignment))
        .route("/remark", post(remark_submissions).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
        .route("/resubmit", post(resubmit_submissions).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
}
