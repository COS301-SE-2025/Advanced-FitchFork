//! Plagiarism routes module.
//!
//! Provides the `/assignments/plagiarism` route group for handling plagiarism cases in assignments.
//!
//! Routes include:
//! - Create, update, delete, and list plagiarism cases
//! - Run MOSS plagiarism checks and get MOSS reports
//! - Flag and review plagiarism cases
//! - Retrieve plagiarism graph for visualization
//!
//! Access control should be enforced via middleware (not shown here) for lecturers, tutors, or assistants.

use axum::{routing::{delete, get, patch, post, put}, Router};
use get::{list_plagiarism_cases, get_graph, get_moss_report};
use post::{create_plagiarism_case, run_moss_check};
use put::update_plagiarism_case;
use delete::{delete_plagiarism_case, bulk_delete_plagiarism_cases};
use patch::{patch_plagiarism_flag, patch_plagiarism_review};

pub mod get;
pub mod post;
pub mod put;
pub mod delete;
pub mod patch;

/// Builds and returns the `/assignments/plagiarism` route group.
///
/// Routes:
/// - `GET    /assignments/plagiarism`                  → List plagiarism cases
/// - `GET    /assignments/plagiarism/graph`            → Get plagiarism graph
/// - `POST   /assignments/plagiarism`                 → Create a new plagiarism case
/// - `POST   /assignments/plagiarism/moss`            → Run MOSS check on submissions
/// - `GET    /assignments/plagiarism/moss`            → Get MOSS report
/// - `PUT    /assignments/plagiarism/{case_id}`       → Update a plagiarism case
/// - `DELETE /assignments/plagiarism/{case_id}`       → Delete a plagiarism case
/// - `DELETE /assignments/plagiarism/bulk`            → Bulk delete plagiarism cases
/// - `PATCH  /assignments/plagiarism/{case_id}/flag`  → Flag a plagiarism case
/// - `PATCH  /assignments/plagiarism/{case_id}/review`→ Review a plagiarism case
pub fn plagiarism_routes() -> Router {
    Router::new()
        .route("/", get(list_plagiarism_cases))
        .route("/graph", get(get_graph))
        .route("/", post(create_plagiarism_case))
        .route("/moss", post(run_moss_check))
        .route("/moss", get(get_moss_report))
        .route("/{case_id}", put(update_plagiarism_case))
        .route("/{case_id}", delete(delete_plagiarism_case))
        .route("/bulk", delete(bulk_delete_plagiarism_cases))
        .route("/{case_id}/flag", patch(patch_plagiarism_flag))
        .route("/{case_id}/review", patch(patch_plagiarism_review))
}
