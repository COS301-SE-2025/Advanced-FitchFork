use axum::{
    Router, routing::{post, delete, put}
};

pub mod post;
pub mod delete;
pub mod put;
use post::create;
use delete::delete_assignment;
use put::edit_assignment;
/// Expects a module ID
/// If an assignment ID is included it will be deleted
/// - `POST /` → `create` 
/// - `DELTE /:assignment_id` → `delete_assignment`
pub fn assignment_routes() -> Router {
    Router::new()
        .route("/", post(create))
        .route("/:assignment_id", delete(delete_assignment))
        .route("/:assignment_id", put(edit_assignment))
}