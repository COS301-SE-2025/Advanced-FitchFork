use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::Value;
use sea_orm::{
    EntityTrait, ColumnTrait, QueryFilter, ActiveModelTrait, Set,
};

use crate::{
    response::ApiResponse,
};

use db::{
    connect,
    models::{
        assignment::{
            Column as AssignmentColumn, Entity as AssignmentEntity,
        },
    },
};

/// POST /api/modules/:module_id/assignments/:assignment_id/config
///
/// Save or replace the JSON configuration object for a specific assignment. Accessible to users with
/// Lecturer or Admin roles assigned to the module.
///
/// This endpoint allows authorized users to attach or update an assignment's configuration object.
/// The configuration contains settings that control various aspects of the assignment evaluation
/// process. The payload must be a valid JSON object.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to set configuration for
///
/// ### Request Body
/// ```json
/// {
///   "test_timeout": 300,
///   "max_memory": "512MB",
///   "allowed_languages": ["java", "python"],
///   "grading_criteria": {
///     "test_weight": 0.7,
///     "style_weight": 0.3
///   },
///   "environment_variables": {
///     "JAVA_HOME": "/usr/lib/jvm/java-11",
///     "PYTHONPATH": "/usr/local/lib/python3.9"
///   }
/// }
/// ```
///
/// ### Request Body Fields
/// The configuration object can contain any valid JSON object with assignment-specific settings.
/// Common fields include:
/// - `test_timeout`: Maximum execution time for tests (in seconds)
/// - `max_memory`: Memory limit for test execution
/// - `allowed_languages`: Array of programming languages allowed for submissions
/// - `grading_criteria`: Object containing various grading weights and criteria
/// - `environment_variables`: Key-value pairs for test environment setup
///
/// ### Example Request
/// ```bash
/// curl -X POST http://localhost:3000/api/modules/1/assignments/2/config \
///   -H "Authorization: Bearer <token>" \
///   -H "Content-Type: application/json" \
///   -d '{
///     "test_timeout": 300,
///     "allowed_languages": ["java", "python"],
///     "grading_criteria": {
///       "test_weight": 0.7,
///       "style_weight": 0.3
///     }
///   }'
/// ```
///
/// ### Success Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "message": "Assignment configuration saved",
///   "data": null
/// }
/// ```
///
/// ### Error Responses
///
/// **400 Bad Request** - Invalid configuration format
/// ```json
/// {
///   "success": false,
///   "message": "Configuration must be a JSON object"
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
///   "message": "Failed to save configuration"
/// }
/// ```
///
/// ### Validation Rules
/// - Request body must be a valid JSON object (not an array, primitive, or null)
/// - Assignment must exist and belong to the specified module
/// - User must have appropriate permissions for the module
///
/// ### Notes
/// - This endpoint completely replaces any existing configuration for the assignment
/// - The configuration object is stored as JSON in the database
/// - Configuration is used by the evaluation system to customize assignment behavior
/// - Only valid JSON objects are accepted; arrays, primitives, or null values are rejected
/// - Configuration setting is restricted to users with appropriate module permissions
pub async fn set_assignment_config(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(config): Json<Value>,
) -> impl IntoResponse {
    if !config.is_object() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("Configuration must be a JSON object")),
        )
            .into_response();
    }

    let db = connect().await;

    let assignment = match AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id as i32))
        .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
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

    let mut active_model: db::models::assignment::ActiveModel = assignment.into();
    active_model.config = Set(Some(config));

    match active_model.update(&db).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::<()>::success((), "Assignment configuration saved")),
        )
            .into_response(),
        Err(err) => {
            eprintln!("Failed to save config: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to save configuration")),
            )
                .into_response()
        }
    }
}