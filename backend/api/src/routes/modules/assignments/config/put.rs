use axum::{
    extract::{State, Path},
    Json,
    http::StatusCode,
    response::IntoResponse,
};
use db::models::{assignment};
use serde::{Deserialize};
use serde_json::{Value};
use sea_orm::{EntityTrait, Set, QueryFilter, ColumnTrait, ActiveModelTrait, DatabaseConnection};
use db::models::assignment::{Column as AssignmentColumn, Entity as AssignmentEntity};
use crate::response::ApiResponse;

#[derive(Debug, Deserialize)]
pub struct PartialConfigUpdate {
    #[serde(flatten)]
    pub fields: serde_json::Map<String, Value>,
}

/// PUT /api/modules/{module_id}/assignments/{assignment_id}/config
///
/// Partially update specific fields of an assignment's configuration. Accessible to users with
/// Lecturer or Admin roles assigned to the module.
///
/// This endpoint merges the provided fields into the existing JSON configuration object, allowing
/// selective updates without replacing the entire configuration. Only specific configuration keys
/// are currently supported for partial updates. Unrecognized keys will result in a validation error.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to update configuration for
///
/// ### Request Body
/// A JSON object containing only the fields to update:
/// ```json
/// {
///   "timeout_seconds": 300,
///   "max_processors": 4
/// }
/// ```
///
/// ### Supported Fields
/// The following fields can be updated via this endpoint:
/// - `timeout_seconds` (integer): Maximum execution time for tests (in seconds)
/// - `max_processors` (integer): Maximum number of processors allowed for test execution
///
/// ### Example Request
/// ```bash
/// curl -X PUT http://localhost:3000/api/modules/1/assignments/2/config \
///   -H "Authorization: Bearer <token>" \
///   -H "Content-Type: application/json" \
///   -d '{
///     "timeout_seconds": 300,
///     "max_processors": 4
///   }'
/// ```
///
/// ### Success Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "message": "Assignment configuration updated successfully",
///   "data": null
/// }
/// ```
///
/// ### Error Responses
///
/// **400 Bad Request** - Validation errors
/// ```json
/// {
///   "success": false,
///   "message": "timeout_seconds must be an integer"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "max_processors must be an integer"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Unknown field: unsupported_field"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Invalid existing config format"
/// }
/// ```
///
/// **403 Forbidden** - Insufficient permissions
/// ```json
/// {
///   "success": false,
///   "message": "Access denied"
/// }
/// ```
///
/// **404 Not Found** - Assignment or module not found
/// ```json
/// {
///   "success": false,
///   "message": "Assignment or module not found"
/// }
/// ```
///
/// **500 Internal Server Error** - Database or server error
/// ```json
/// {
///   "success": false,
///   "message": "Failed to update configuration"
/// }
/// ```
///
/// ### Validation Rules
/// - Request body must be a valid JSON object
/// - Only supported fields (`timeout_seconds`, `max_processors`) are allowed
/// - Field values must be integers for both supported fields
/// - Assignment must exist and belong to the specified module
/// - User must have appropriate permissions for the module
/// - Existing configuration must be a valid JSON object (if present)
///
/// ### Notes
/// - This endpoint performs a partial update, merging new values with existing configuration
/// - Only the specified fields are updated; other configuration fields remain unchanged
/// - If no configuration exists, a new configuration object is created with the provided fields
/// - For complete configuration replacement, use the POST endpoint instead
/// - Partial updates are restricted to users with appropriate module permissions
/// - The `timeout_seconds` and `max_processors` fields are validated to ensure they are integers
pub async fn update_assignment_config(
    State(db): State<DatabaseConnection>,
    Path((module_id, assignment_id)) : Path<(i64, i64)>,
    Json(payload): Json<PartialConfigUpdate>
) -> impl IntoResponse {
    let assignment = match AssignmentEntity::find()
        .filter(AssignmentColumn::ModuleId.eq(module_id))
        .filter(AssignmentColumn::Id.eq(assignment_id))
        .one(&db)
        .await
    {
        Ok(Some(a)) => a,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Assignment or module not found")),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error")),
            )
                .into_response();
        }
    };

    let config = assignment.config.clone();
    let mut existing_config: serde_json::Map<String, Value> = match config {
        Some(Value::Object(obj)) => obj,
        Some(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("Invalid existing config format")),
            )
                .into_response();
        }
        None => serde_json::Map::new(),
    };

    for (key, value) in &payload.fields {
        match key.as_str() {
            "timeout_seconds" => {
                if !value.is_u64() && !value.is_i64() {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::<()>::error("timeout_seconds must be an integer")),
                    )
                        .into_response();
                }
            }
            "max_processors" => {
                if !value.is_u64() && !value.is_i64() {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::<()>::error("max_processors must be an integer")),
                    )
                        .into_response();
                }
            }
            _ => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<()>::error(&format!("Unknown field: {}", key))),
                )
                    .into_response();
            }
        }
    }

    existing_config.extend(payload.fields.clone());
    let mut assignment_model: assignment::ActiveModel = assignment.into();
    assignment_model.config = Set(Some(Value::Object(existing_config)));

    if let Err(_) = assignment_model.update(&db).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to update configuration")),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(ApiResponse::<()>::success((), "Assignment configuration updated successfully")),
    )
        .into_response()
}