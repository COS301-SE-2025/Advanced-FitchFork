use crate::auth::guards::allow_lecturer;
use axum::{
    Router,
    middleware::from_fn,
    routing::{delete, get, post},
};

pub mod delete;
pub mod get;
pub mod post;

use delete::delete_task_overwrite_files;
use get::get_task_overwrite_files;
use post::post_task_overwrite_files;

/// Routes under `/assignments/{assignment_id}/overwrite_files`:
///
/// Provides endpoints to manage overwrite files for assignment tasks.  
/// All routes require lecturer permissions.
///
/// ### Routes:
/// - `GET /task/{task_id}` → Retrieve all overwrite files for a specific task.  
///   Returns the actual file content(s) or metadata.
///
/// - `POST /task/{task_id}` → Upload one or more overwrite files for a specific task.  
///   Accepts multipart/form-data or JSON (depending on implementation).
///
/// - `DELETE /task/{task_id}` → Delete all overwrite files for the specific task.
pub fn overwrite_file_routes() -> Router {
    Router::new()
        .route(
            "/task/{task_id}",
            get(get_task_overwrite_files)
                .route_layer(from_fn_with_state(app_state.clone(), allow_lecturer)),
        )
        .route(
            "/task/{task_id}",
            delete(delete_task_overwrite_files)
                .route_layer(from_fn_with_state(app_state.clone(), allow_lecturer)),
        )
        .route(
            "/task/{task_id}",
            post(post_task_overwrite_files)
                .route_layer(from_fn_with_state(app_state.clone(), allow_lecturer)),
        )
}
