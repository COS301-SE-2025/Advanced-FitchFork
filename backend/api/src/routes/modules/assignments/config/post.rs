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

/// POST /assignments/:assignment_id/config
///
/// Save or replace the JSON configuration object for a specific assignment.
///
/// This endpoint allows authorized users (e.g., lecturers or admins) to attach or update
/// an assignment's configuration. The payload must be a valid JSON object.
///
/// ### JSON Body
/// ```json
/// {
///   "timeout_seconds": 10,
///   "allowed_imports": ["os", "sys", "math"],
///   "max_processors": 2,
///   "max_memory_mb": 256,
///   "languages": "python"
/// }
/// ```
///
/// ### Example curl
/// ```bash
/// curl -X POST http://localhost:3000/assignments/1/config \
///   -H "Authorization: Bearer <token>" \
///   -H "Content-Type: application/json" \
///   -d '{"timeout_seconds":10,"languages":"python"}'
/// ```
///
/// ### Responses
/// - `200 OK` if configuration is saved successfully
/// - `400 Bad Request` if payload is not a JSON object
/// - `404 Not Found` if assignment or module does not exist
/// - `500 Internal Server Error` on database failure
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