use axum::{
    Json,
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use util::filters::FilterParam;
use services::service::Service;
use services::assignment_interpreter::AssignmentInterpreterService;

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
    Path((_module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    // Find the interpreter(s) for this assignment
    let interpreters = match AssignmentInterpreterService::find_all(
        &vec![
            FilterParam::eq("assignment_id", assignment_id),
        ],
        &vec![],
        None,
    ).await {
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

    for interpreter in interpreters {
        AssignmentInterpreterService::delete_file_only(interpreter.id).await;
        AssignmentInterpreterService::delete_by_id(interpreter.id).await;
    }

    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "message": "Interpreter removed successfully"
        })),
    )
}
