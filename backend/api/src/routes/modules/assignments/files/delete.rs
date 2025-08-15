use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{
    ActiveModelTrait,
    ColumnTrait,
    EntityTrait,
    QueryFilter,
};
use serde_json::json;
use db::models::assignment_file;

/// DELETE /api/modules/{module_id}/assignments/{assignment_id}/files
///
/// Delete one or more files from a specific assignment. Only accessible by lecturers assigned to the module.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment whose files are to be deleted
///
/// ### Request Body (JSON)
/// - `file_ids` (array of i64): List of file IDs to delete. Must be a non-empty array.
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "message": "Files removed successfully"
/// }
/// ```
///
/// - `400 Bad Request`
/// ```json
/// {
///   "success": false,
///   "message": "Request must include a non-empty list of file_ids"
/// }
/// ```
///
/// - `404 Not Found`
/// ```json
/// {
///   "success": false,
///   "message": "No assignment found with ID <assignment_id> in module <module_id>"
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
///
pub async fn delete_files(
    Path((_, assignment_id)): Path<(i64, i64)>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = db::get_connection().await;

    let file_ids: Vec<i64> = req
        .get("file_ids")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
        .unwrap_or_default();

    if file_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "message": "Request must include a non-empty list of file_ids"
            })),
        );
    }

    let found_models = match assignment_file::Entity::find()
        .filter(assignment_file::Column::AssignmentId.eq(assignment_id as i32))
        .filter(assignment_file::Column::Id.is_in(
            file_ids.iter().copied().map(|id| id as i32).collect::<Vec<_>>(),
        ))
        .all(db)
        .await
    {
        Ok(models) => models,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "success": false,
                    "message": format!("File(s) not found: {:?}", file_ids)
                })),
            );
        }
    };

    let found_ids: Vec<i64> = found_models.iter().map(|m| m.id as i64).collect();
    let missing: Vec<i64> = file_ids
        .iter()
        .filter(|id| !found_ids.contains(id))
        .copied()
        .collect();

    if !missing.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "message": format!("File(s) not found: {:?}", missing)
            })),
        );
    }

    for file in found_models {
        let _ = file.delete_file_only();
        let am: assignment_file::ActiveModel = file.into();
        let _ = am.delete(db).await;
    }

    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "message": "Files removed successfully"
        })),
    )
}