use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use db::{
    models::{assignment::Assignment, assignment_files::AssignmentFiles},
    pool,
};
use serde_json::json;
///   Deletes a specific assignment and its associated files.
///
/// #Arguements are passed in based on method route automatically
///
/// Returns an HTTP response indicating the result:
/// - `200 OK` with a success message if the assignment was found and deleted.
/// - `404 NOT FOUND` if no assignment was found for the given `module_id` and `assignment_id`.
/// - `500 INTERNAL SERVER ERROR` if a database or internal error occurred, including the error message.

pub async fn delete_assignment(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    match Assignment::delete_by_id(Some(pool::get()), assignment_id, module_id).await {
        Ok(true) => {
            return (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "message": format!("Assignment ID {} and attached files deleted successfully", assignment_id)
                })),
            );
        }
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

pub async fn delete_all_files(pool: &sqlx::SqlitePool, files: Vec<i64>, assignment_id: i64) {
    for f in files {
        if let Ok(Some(_)) = sqlx::query_scalar::<_, i32>(
            "SELECT 1 FROM assignment_files WHERE id = ? AND assignment_id = ?",
        )
        .bind(f)
        .bind(assignment_id)
        .fetch_optional(&*pool)
        .await
        {
            let _ = AssignmentFiles::delete_by_id(Some(pool), f).await;
        }
    }
}

pub async fn delete_files(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let pool = pool::get();
    let res =
        sqlx::query_scalar::<_, i32>("SELECT 1 FROM assignments WHERE id = ? AND module_id = ?")
            .bind(assignment_id)
            .bind(module_id)
            .fetch_optional(&*pool)
            .await;
    if let Ok(None) = res {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "message": format!("No assignment found with ID {} in module {}", assignment_id, module_id)
            })),
        );
    } else if let Err(e) = res {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "message": format!("Database error: {}", e)
            })),
        );
    }

    let files: Vec<i64> = req
        .get("file_ids")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|val| val.as_i64()).collect())
        .unwrap_or_default();

    if files.len() == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
            "success": false,
            "message": "Request must include a non-empty list of file_ids"
        })),
        );
    }
    delete_all_files(pool, files, assignment_id).await;

    return (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "message": "Files removed successfully"
        })),
    );
}
