//! Module deletion routes.
//!
//! Provides endpoints to delete modules:
//! - `DELETE /api/modules/{id}` → Permanently delete a single module by ID.
//! - `DELETE /api/modules/bulk` → Bulk delete multiple modules by their IDs.
//!
//! All responses follow the standard `ApiResponse` format.

use crate::response::ApiResponse;
use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use services::module::ModuleService;
use services::service::Service;
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
pub async fn delete_module(Path(module_id): Path<i64>) -> impl IntoResponse {
    match ModuleService::delete_by_id(module_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::<()>::success(
                (),
                "Module deleted successfully",
            )),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!(
                "Failed to delete module: {}",
                e
            ))),
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
pub async fn bulk_delete_modules(Json(req): Json<BulkDeleteRequest>) -> impl IntoResponse {
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
        match ModuleService::delete_by_id(id).await {
            Ok(_) => deleted_count += 1,
            Err(e) => {
                failed.push(FailedDelete {
                    id,
                    error: format!("Failed to delete module: {}", e),
                });
            }
        }
    }

    let result = BulkDeleteResult {
        deleted: deleted_count,
        failed,
    };

    let message = format!("Deleted {}/{} modules", deleted_count, req.module_ids.len());

    (StatusCode::OK, Json(ApiResponse::success(result, message)))
}
