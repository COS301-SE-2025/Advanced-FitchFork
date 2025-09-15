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

use axum::{
    middleware::from_fn_with_state,
    routing::{get, post, delete, patch},
    Router,
};

use get::{get_submission, list_submissions, get_submission_output};
use post::{submit_assignment, remark_submissions, resubmit_submissions};
use delete::{delete_submission, bulk_delete_submissions};
use patch::{set_submission_ignored};

use util::state::AppState;
use crate::auth::guards::{
    require_lecturer_or_assistant_lecturer,
    require_lecturer_or_tutor,
    require_ready_assignment,
};
use crate::routes::modules::assignments::submissions::get::download_submission_file;

pub mod common;
pub mod get;
pub mod post;
pub mod delete;
pub mod patch;

/// Build the `/submissions` routes for a specific assignment.
///
/// ### Routes
/// - `GET    /`                          — List submissions (students: only their own; staff: all)
/// - `GET    /{submission_id}`           — Get a submission's report (role-aware extras)
/// - `GET    /{submission_id}/output`    — Get task output (**lecturer/tutor only**)
/// - `GET    /{submission_id}/download`  — Download original submission file (owner or staff)
/// - `POST   /`                          — Create a submission (**student only**, guarded by assignment readiness)
/// - `POST   /remark`                    — Remark submissions (**lecturer/assistant lecturer only**)
/// - `POST   /resubmit`                  — Resubmit submissions (**lecturer/assistant lecturer only**)
/// - `PATCH  /{submission_id}/ignore`    — Toggle `ignored` flag (**lecturer/assistant lecturer only**)
/// - `DELETE /{submission_id}`           — Delete a submission (**lecturer/assistant lecturer only**)
/// - `DELETE /bulk`                      — Bulk delete submissions (**lecturer/assistant lecturer only**)
pub fn submission_routes() -> Router {
    Router::new()
        .route("/", get(list_submissions))
        .route("/{submission_id}", get(get_submission))
        .route("/{submission_id}/output", get(get_submission_output).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_tutor)))
        .route("/{submission_id}/download", get(download_submission_file))
        .route("/{submission_id}",delete(delete_submission).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
        .route("/bulk",delete(bulk_delete_submissions).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
        .route("/", post(submit_assignment).route_layer(from_fn_with_state(app_state.clone(), require_ready_assignment)))
        .route("/remark", post(remark_submissions).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
        .route("/resubmit", post(resubmit_submissions).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
        .route("/{submission_id}/ignore", patch(set_submission_ignored).route_layer(from_fn_with_state(app_state.clone(), require_lecturer_or_assistant_lecturer)))
}
