use axum::{Router, routing::{get, post}};
use sea_orm::DatabaseConnection;
use get::{list_submissions, get_submission};
use post::{submit_assignment, remark_submissions};

pub mod get;
pub mod post;
pub mod common;

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
pub fn submission_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/", get(list_submissions))
        .route("/{submission_id}", get(get_submission))
        .route("/", post(submit_assignment))
        .route("/remark", post(remark_submissions))
}