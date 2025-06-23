use axum::{
    extract::Path,
    Json,
    http::StatusCode,
    response::IntoResponse,
};
use db::models::{assignment};
use serde::{Deserialize};
use serde_json::{Value};
use sea_orm::{EntityTrait, Set, QueryFilter, ColumnTrait, ActiveModelTrait};

use db::{
    connect,
    models::{
        assignment::{
            Column as AssignmentColumn, Entity as AssignmentEntity,
        },
    },
};

use crate::response::ApiResponse;


#[derive(Debug, Deserialize)]
pub struct PartialConfigUpdate {
    #[serde(flatten)]
    pub fields: serde_json::Map<String, Value>,
}
/// PUT /assignments/:assignment_id/config
///
/// Partially update specific fields of an assignment's configuration.
///
/// This endpoint merges the provided fields into the existing JSON configuration,
/// validating known keys. Only a subset of configuration keys are currently supported.
/// Unrecognized keys will result in a `400 Bad Request`.
///
/// ### JSON Body
/// A JSON object containing only the fields to update.
/// ```json
/// {
///   "timeout_seconds": 20,
///   "max_processors": 4
/// }
/// ```
///
/// ### Allowed Fields
/// - `timeout_seconds` (integer)
/// - `max_processors` (integer)
///
/// ### Example curl
/// ```bash
/// curl -X PUT http://localhost:3000/assignments/1/config \
///   -H "Authorization: Bearer <token>" \
///   -H "Content-Type: application/json" \
///   -d '{"timeout_seconds": 20, "max_processors": 4}'
/// ```
///
/// ### Responses
/// - `200 OK` if the config was updated successfully
/// - `400 Bad Request` if the request includes unknown or invalid fields
/// - `404 Not Found` if the assignment or module does not exist
/// - `500 Internal Server Error` on database failure
pub async fn update_assignment_config(Path((module_id, assignment_id)) : Path<(i64, i64)>, Json(payload): Json<PartialConfigUpdate>) -> impl IntoResponse {
    let db = connect().await;

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