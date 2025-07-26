use axum::{
    extract::{State, Path},
    Json,
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::{Value, Map};
use crate::{
    response::ApiResponse,
};
use db::{
    models::{
        assignment::{
            Column as AssignmentColumn, Entity as AssignmentEntity,
        },
    },
};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, DatabaseConnection};

/// GET /api/modules/{module_id}/assignments/{assignment_id}/config
///
/// Retrieve the JSON configuration object associated with a specific assignment. Accessible to users
/// assigned to the module with appropriate permissions.
///
/// The configuration object contains assignment-specific settings that control various aspects of
/// the assignment evaluation process, such as test parameters, grading criteria, or execution settings.
/// If no configuration has been set, an empty JSON object is returned.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to retrieve configuration for
///
/// ### Example Request
/// ```bash
/// curl -X GET http://localhost:3000/api/modules/1/assignments/2/config \
///   -H "Authorization: Bearer <token>"
/// ```
///
/// ### Success Response (200 OK) - With Configuration
/// ```json
/// {
///   "success": true,
///   "message": "Assignment configuration retrieved successfully",
///   "data": {
///     "test_timeout": 300,
///     "max_memory": "512MB",
///     "allowed_languages": ["java", "python"],
///     "grading_criteria": {
///       "test_weight": 0.7,
///       "style_weight": 0.3
///     }
///   }
/// }
/// ```
///
/// ### Success Response (200 OK) - No Configuration Set
/// ```json
/// {
///   "success": true,
///   "message": "No configuration set for this assignment",
///   "data": {}
/// }
/// ```
///
/// ### Error Responses
///
/// **400 Bad Request** - Invalid configuration format
/// ```json
/// {
///   "success": false,
///   "message": "Invalid configuration format"
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
/// **500 Internal Server Error** - Database error
/// ```json
/// {
///   "success": false,
///   "message": "Database error"
/// }
/// ```
///
/// ### Configuration Format
/// The configuration must be a valid JSON object. Common configuration fields include:
/// - `test_timeout`: Maximum execution time for tests (in seconds)
/// - `max_memory`: Memory limit for test execution
/// - `allowed_languages`: Array of programming languages allowed for submissions
/// - `grading_criteria`: Object containing various grading weights and criteria
/// - `environment_variables`: Key-value pairs for test environment setup
///
/// ### Notes
/// - Configuration is optional; assignments can function without a configuration object
/// - The configuration object is used by the evaluation system to customize assignment behavior
/// - Only valid JSON objects are accepted as configuration; arrays, primitives, or null values are rejected
/// - Configuration retrieval is restricted to users with appropriate module permissions
pub async fn get_assignment_config(
    State(db): State<DatabaseConnection>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let assignment = AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id as i32))
        .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
        .one(&db).await.unwrap().unwrap();

    let config = match assignment.config {
        Some(Value::Object(obj)) => Value::Object(obj),
        Some(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("Invalid configuration format")),
            )
                .into_response();
        }
        None => Value::Object(Map::new()),
    };

    let message = if config.as_object().unwrap().is_empty() {
        "No configuration set for this assignment"
    } else {
        "Assignment configuration retrieved successfully"
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(config, message)),
    )
        .into_response()
}