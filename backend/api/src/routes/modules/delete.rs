use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::response::ApiResponse;

use db::{
    connect,
    models::module
};

/// DELETE /api/modules/:module_id
///
/// Permanently deletes a module by ID, including all its assignments and assignment files.  
/// Only accessible by admin users.
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": null,
///   "message": "Module deleted successfully"
/// }
/// ```
///
/// - `403 Forbidden`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "You do not have permission to perform this action"
/// }
/// ```
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Module not found"
/// }
/// ```
pub async fn delete_module(Path(module_id): Path<i32>) -> impl IntoResponse {
    let db = connect().await;

    match module::Entity::find()
        .filter(module::Column::Id.eq(module_id))
        .one(&db)
        .await
    {
        Ok(Some(m)) => {
            match m.delete(&db).await {
                Ok(_) => (
                    StatusCode::OK,
                    Json(ApiResponse::<()>::success((), "Module deleted successfully")),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(format!("Failed to delete module: {}", e))),
                ),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
        ),
    }
}