use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use db::models::assignment_interpreter;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;
use util::state::AppState;

/// DELETE /api/modules/{module_id}/assignments/{assignment_id}/interpreter
///
/// Delete the interpreter for a specific assignment. Only accessible by lecturers assigned to the module.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment whose interpreter is to be deleted
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "message": "Interpreter removed successfully"
/// }
/// ```
///
/// - `404 Not Found`
/// ```json
/// {
///   "success": false,
///   "message": "No interpreter found for assignment <assignment_id>"
/// }
/// ```
///
/// - `500 Internal Server Error`
/// ```json
/// {
///   "success": false,
///   "message": "<database error details>"
/// }
/// ```
pub async fn delete_interpreter(
    State(app_state): State<AppState>,
    Path((_module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let db = app_state.db();

    // Find the interpreter(s) for this assignment
    let interpreters = match assignment_interpreter::Entity::find()
        .filter(assignment_interpreter::Column::AssignmentId.eq(assignment_id))
        .all(db)
        .await
    {
        Ok(models) => models,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "message": format!("Database error: {:?}", err)
                })),
            );
        }
    };

    if interpreters.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "message": format!("No interpreter found for assignment {}", assignment_id)
            })),
        );
    }

    // Delete each interpreter file and DB record
    for interpreter in interpreters {
        let _ = interpreter.delete_file_only(); // ignore file deletion errors
        let am: assignment_interpreter::ActiveModel = interpreter.into();
        let _ = am.delete(db).await; // ignore DB deletion errors for now
    }

    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "message": "Interpreter removed successfully"
        })),
    )
}
