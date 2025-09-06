use crate::auth::guards::require_lecturer;
use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{delete, get, post},
};
use util::state::AppState;

pub mod delete;
pub mod get;
pub mod post;

use delete::delete_task_overwrite_files;
use get::get_task_overwrite_files;
use post::post_task_overwrite_files;

/// Routes under `/assignments/{assignment_id}/overwrite_files`:
///
/// - `POST /task/{task_number}` → Upload one or more overwrite files for a specific task.  
///

//NOTE - WHY DOES THIS NEED TO BE task_id AND CANT BE task_number???
pub fn overwrite_file_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/task/{task_id}",
            get(get_task_overwrite_files)
                .route_layer(from_fn_with_state(app_state.clone(), require_lecturer)),
        )
        .route(
            "/task/{task_id}",
            delete(delete_task_overwrite_files)
                .route_layer(from_fn_with_state(app_state.clone(), require_lecturer)),
        )
        .route(
            "/task/{task_id}",
            post(post_task_overwrite_files)
                .route_layer(from_fn_with_state(app_state.clone(), require_lecturer)),
        )
}
