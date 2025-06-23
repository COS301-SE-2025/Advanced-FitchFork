use axum::{
    extract::Path,
    Json,
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::{Value, Map};

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
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};

/// GET /assignments/:assignment_id/config
///
/// Retrieve the JSON configuration object associated with a specific assignment.
///
/// The configuration is returned only if it exists and is a valid JSON object. If no configuration
/// has been set, an empty object is returned with an appropriate message.
///
/// ### Example curl
/// ```bash
/// curl -X GET http://localhost:3000/assignments/1/config \
///   -H "Authorization: Bearer <token>"
/// ```
///
/// ### Responses
/// - `200 OK` with the JSON configuration object (empty object if not set)
/// - `400 Bad Request` if the stored config is not a valid JSON object
/// - `404 Not Found` if the assignment or module does not exist
/// - `500 Internal Server Error` on database failure
pub async fn get_assignment_config( Path((module_id, assignment_id)): Path<(i64, i64)>,) -> impl IntoResponse {

    let db = connect().await;

    let result = AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id as i32))
        .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
        .one(&db)
        .await;

    let assignment = match result {
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
