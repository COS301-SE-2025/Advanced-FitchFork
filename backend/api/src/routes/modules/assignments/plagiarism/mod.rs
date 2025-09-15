//! Plagiarism routes module.
//!
//! Provides the `/assignments/plagiarism` route group for handling plagiarism cases in assignments.
//!
//! Routes include:
//! - Create, update, delete, and list plagiarism cases
//! - Run MOSS plagiarism checks and list MOSS reports
//! - Flag and review plagiarism cases
//! - Retrieve plagiarism graph for visualization
//! - Manage versioned MOSS archives (create, delete, **download specific report**)
//! - List stored MOSS reports from the database
//!
//! Access control should be enforced via middleware (not shown here) for lecturers, tutors, or assistants.

use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};

use get::{get_graph, list_moss_reports, list_plagiarism_cases, download_moss_archive_by_report};
use post::{create_plagiarism_case, run_moss_check};
use put::update_plagiarism_case;
use delete::{bulk_delete_plagiarism_cases, delete_plagiarism_case};
use patch::{patch_plagiarism_flag, patch_plagiarism_review};
use util::state::AppState;

use crate::routes::modules::assignments::plagiarism::delete::delete_moss_report;

pub mod get;
pub mod post;
pub mod put;
pub mod delete;
pub mod patch;

/// Builds and returns the `/assignments/plagiarism` route group.
///
/// Routes:
/// - `GET    /assignments/plagiarism`                               → List plagiarism cases
/// - `GET    /assignments/plagiarism/graph`                         → Get plagiarism graph
/// - `POST   /assignments/plagiarism`                               → Create a new plagiarism case
/// - `PUT    /assignments/plagiarism/{case_id}`                     → Update a plagiarism case
/// - `DELETE /assignments/plagiarism/{case_id}`                     → Delete a plagiarism case
/// - `DELETE /assignments/plagiarism/bulk`                          → Bulk delete plagiarism cases
/// - `PATCH  /assignments/plagiarism/{case_id}/flag`                → Flag a plagiarism case
/// - `PATCH  /assignments/plagiarism/{case_id}/review`              → Review a plagiarism case
/// - `POST   /assignments/plagiarism/moss`                          → Run MOSS check (also kicks off a versioned archive job)
/// - `GET    /assignments/plagiarism/moss/reports`                  → List stored MOSS reports (from DB)
/// - `GET    /assignments/plagiarism/moss/reports/{report_id}/download` → Download the archive ZIP for a **specific** report
/// - `DELETE /assignments/plagiarism/moss/reports/{report_id}`      → Delete a specific moss report
pub fn plagiarism_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_plagiarism_cases))
        .route("/graph", get(get_graph))
        .route("/", post(create_plagiarism_case))
        .route("/{case_id}", put(update_plagiarism_case))
        .route("/{case_id}", delete(delete_plagiarism_case))
        .route("/bulk", delete(bulk_delete_plagiarism_cases))
        .route("/{case_id}/flag", patch(patch_plagiarism_flag))
        .route("/{case_id}/review", patch(patch_plagiarism_review))
        .route("/moss", post(run_moss_check))
        .route("/moss/reports", get(list_moss_reports))
        .route("/moss/reports/{report_id}/download", get(download_moss_archive_by_report)) 
        .route("/moss/reports/{report_id}", delete(delete_moss_report))
}
