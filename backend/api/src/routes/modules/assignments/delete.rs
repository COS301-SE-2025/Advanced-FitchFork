use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use db::{models::assignment::Assignment, pool};
use serde_json::json;

pub async fn delete_assignment(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    match Assignment::delete_by_id(Some(pool::get()), assignment_id, module_id).await {
        Ok(true) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "message": format!("Assignment ID {} and attached files deleted successfully", assignment_id)
            })),
        ),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "message": format!("No assignment found with ID {} in module {}", assignment_id, module_id)
            })),
        ),
        Err(e) => {
            let message = if let Some(db_err) = e.as_database_error() {
                db_err.message().to_string()
            } else {
                "An unknown error occurred".to_string()
            };
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "message": message
                })),
            )
        }
    }
}
