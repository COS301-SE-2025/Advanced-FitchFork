use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use sea_orm::{
    ActiveModelTrait,
    ColumnTrait,
    DbErr,
    EntityTrait,
    QueryFilter,
};

use serde_json::json;

use db::{
    connect,
    models::{assignment, assignment_file},
};

/// Deletes a specific assignment and its associated files and folder.
///
/// # Path parameters
/// - `module_id`: ID of the module
/// - `assignment_id`: ID of the assignment to delete
///
/// # Returns
/// - `200 OK` if deletion succeeded
/// - `404 NOT FOUND` if assignment not found
/// - `500 INTERNAL SERVER ERROR` on DB or FS error
pub async fn delete_assignment(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let db = connect().await;

    match assignment::Model::delete(&db, assignment_id as i32, module_id as i32).await {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "message": format!("Assignment {} deleted successfully", assignment_id),
            })),
        ),
        Err(DbErr::RecordNotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "message": format!("No assignment found with ID {} in module {}", assignment_id, module_id),
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "message": e.to_string(),
            })),
        ),
    }
}

/// Deletes a list of files associated with a specific assignment.
async fn delete_all_files(
    db: &sea_orm::DatabaseConnection,
    file_ids: Vec<i64>,
    assignment_id: i64,
) {
    for file_id in file_ids {
        if let Ok(Some(file)) = assignment_file::Entity::find()
            .filter(assignment_file::Column::Id.eq(file_id as i32))
            .filter(assignment_file::Column::AssignmentId.eq(assignment_id as i32))
            .one(db)
            .await
        {
            let _ = file.delete_file_only();
            let model: assignment_file::ActiveModel = file.into();
            let _ = model.delete(db).await;
        }
    }
}

/// Deletes selected files from an assignment.
pub async fn delete_files(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let db = connect().await;

    match assignment::Entity::find()
        .filter(assignment::Column::Id.eq(assignment_id as i32))
        .filter(assignment::Column::ModuleId.eq(module_id as i32))
        .one(&db)
        .await
    {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "success": false,
                    "message": format!("No assignment found with ID {} in module {}", assignment_id, module_id)
                })),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "message": e.to_string()
                })),
            );
        }
    }

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

    delete_all_files(&db, file_ids, assignment_id).await;

    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "message": "Files removed successfully"
        })),
    )
}