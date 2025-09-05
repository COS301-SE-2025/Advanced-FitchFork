//! Submission routes module.
//!
//! Provides the `/submissions` route group for handling assignment submissions.
//!
//! Routes include:
//! - Create, resubmit, remark, get, and download submissions
//! - List all submissions
//! - Get submission output
//!
//! Access control is enforced via middleware guards for students, tutors, or lecturers.

use axum::{middleware::from_fn_with_state, routing::{get, post}, Router};
use get::{get_submission, list_submissions, get_submission_output};
use post::{submit_assignment, remark_submissions, resubmit_submissions};
use util::state::AppState;
use crate::{auth::guards::{require_lecturer_or_assistant_lecturer, require_lecturer_or_tutor, require_ready_assignment}, routes::modules::assignments::submissions::get::download_submission_file};

pub mod common;
pub mod get;
pub mod post;

/// Builds and returns the `/submissions` route group for a given assignment context.
///
/// Routes:
/// - `GET    /`                          → List all submissions (restricted by role)
/// - `GET    /{submission_id}`           → Get details of a specific submission
/// - `GET    /{submission_id}/output`    → Get submission output (lecturer/tutor only)
/// - `GET    /{submission_id}/download`  → Download the original submission file
/// - `POST   /`                          → Submit a new assignment (student only)
/// - `POST   /remark`                    → Remark submissions (lecturer/assistant lecturer only)
/// - `POST   /resubmit`                  → Resubmit submissions (lecturer/assistant lecturer only)
pub fn submission_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_submissions))
        .route("/{submission_id}", get(get_submission))
        .route("/{submission_id}/output", get(get_submission_output).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_tutor)))
        .route("/{submission_id}/download", get(download_submission_file))
        .route("/", post(submit_assignment).route_layer(from_fn_with_state(app_state.clone(), require_ready_assignment)))
        .route("/remark", post(remark_submissions).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
        .route("/resubmit", post(resubmit_submissions).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
}
