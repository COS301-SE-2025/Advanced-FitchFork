use axum::{
    Router, routing::{post, delete}
};

pub mod post;
pub mod delete;

use post::create;
use delete::delete_assignment;
/// Expects a module ID
/// If an assignment ID is included it will be deleted
/// - `POST /` → `create` 
/// - `DELTE /:assignment_id` → `delete_assignment`
pub fn assignment_routes() -> Router {
    Router::new()
        .route("/", post(create))
        .route("/:assignment_id", delete(delete_assignment))
}