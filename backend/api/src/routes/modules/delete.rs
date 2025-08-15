use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use crate::response::ApiResponse;
use db::models::module;
use serde::{Serialize, Deserialize};
use validator::Validate;

/// DELETE /api/modules/{module_id}
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
pub async fn delete_module(
    Path(module_id): Path<i64>
) -> impl IntoResponse {
    let db = db::get_connection().await;

    match module::Entity::find()
        .filter(module::Column::Id.eq(module_id))
        .one(db)
        .await
    {
        Ok(Some(m)) => {
            match m.delete(db).await {
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

#[derive(Debug, Deserialize, Validate)]
pub struct BulkDeleteRequest {
    #[validate(length(min = 1, message = "At least one module ID is required"))]
    pub module_ids: Vec<i64>,
}

#[derive(Serialize)]
pub struct BulkDeleteResult {
    pub deleted: usize,
    pub failed: Vec<FailedDelete>,
}

#[derive(Serialize)]
pub struct FailedDelete {
    pub id: i64,
    pub error: String,
}

/// DELETE /api/modules/bulk
///
/// Bulk delete multiple modules by their IDs.
/// Only accessible by admin users.
///
/// ### Request Body
/// ```json
/// {
///   "module_ids": [1, 2, 3]
/// }
/// ```
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "data": {
///     "deleted": 2,
///     "failed": [
///       { "id": 3, "error": "Module not found" }
///     ]
///   },
///   "message": "Deleted 2/3 modules"
/// }
/// ```
///
/// - `400 Bad Request`
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "At least one module ID is required"
/// }
/// ```
pub async fn bulk_delete_modules(
    Json(req): Json<BulkDeleteRequest>,
) -> impl IntoResponse {
    let db = db::get_connection().await;

    // Validate the request
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<BulkDeleteResult>::error(error_message)),
        );
    }

    let mut deleted_count = 0;
    let mut failed = Vec::new();

    for &id in &req.module_ids {
        match module::Entity::find()
            .filter(module::Column::Id.eq(id))
            .one(db)
            .await
        {
            Ok(Some(module_model)) => {
                match module_model.delete(db).await {
                    Ok(_) => deleted_count += 1,
                    Err(e) => {
                        failed.push(FailedDelete {
                            id,
                            error: format!("Failed to delete module: {}", e),
                        });
                    }
                }
            }
            Ok(None) => {
                failed.push(FailedDelete {
                    id,
                    error: "Module not found".into(),
                });
            }
            Err(e) => {
                failed.push(FailedDelete {
                    id,
                    error: format!("Database error: {}", e),
                });
            }
        }
    }

    let result = BulkDeleteResult {
        deleted: deleted_count,
        failed,
    };

    let message = format!(
        "Deleted {}/{} modules",
        deleted_count,
        req.module_ids.len()
    );

    (
        StatusCode::OK,
        Json(ApiResponse::success(result, message)),
    )
}