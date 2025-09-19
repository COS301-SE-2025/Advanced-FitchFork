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
    Router,
    middleware::from_fn_with_state,
    routing::{delete, get, patch, post},
};

use delete::{bulk_delete_submissions, delete_submission};
use get::{get_submission, get_submission_output, list_submissions};
use patch::set_submission_ignored;
use post::{remark_submissions, resubmit_submissions, submit_assignment};

use crate::auth::guards::{allow_assistant_lecturer, allow_ready_assignment, allow_tutor};
use crate::routes::modules::assignments::submissions::get::download_submission_file;
use util::state::AppState;

pub mod common;
pub mod delete;
pub mod get;
pub mod patch;
pub mod post;

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
pub fn submission_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_submissions))
        .route("/{submission_id}", get(get_submission))
        .route(
            "/{submission_id}/output",
            get(get_submission_output)
                .route_layer(from_fn_with_state(app_state.clone(), allow_tutor)),
        )
        .route("/{submission_id}/download", get(download_submission_file))
        .route(
            "/{submission_id}",
            delete(delete_submission).route_layer(from_fn_with_state(
                app_state.clone(),
                allow_assistant_lecturer,
            )),
        )
        .route(
            "/bulk",
            delete(bulk_delete_submissions).route_layer(from_fn_with_state(
                app_state.clone(),
                allow_assistant_lecturer,
            )),
        )
        .route(
            "/",
            post(submit_assignment).route_layer(from_fn_with_state(
                app_state.clone(),
                allow_ready_assignment,
            )),
        )
        .route(
            "/remark",
            post(remark_submissions).route_layer(from_fn_with_state(
                app_state.clone(),
                allow_assistant_lecturer,
            )),
        )
        .route(
            "/resubmit",
            post(resubmit_submissions).route_layer(from_fn_with_state(
                app_state.clone(),
                allow_assistant_lecturer,
            )),
        )
        .route(
            "/{submission_id}/ignore",
            patch(set_submission_ignored).route_layer(from_fn_with_state(
                app_state.clone(),
                allow_assistant_lecturer,
            )),
        )
}
