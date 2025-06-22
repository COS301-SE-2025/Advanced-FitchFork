use axum::{
    extract::Path,
    Json,
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_value, json};
use sea_orm::{EntityTrait, Set};

use crate::{
    db::connect,
    entities::assignment::{Entity as AssignmentEntity, Column as AssignmentColumn, Model as AssignmentModel, ActiveModel},
    response::ApiResponse,
    types::Empty,
};

#[derive(Debug, Deserialize)]
pub struct PartialConfigUpdate {
    #[serde(flatten)]
    pub fields: serde_json::Map<String, Value>,
}


// todo - Add docs
pub async fn update_assignment_config(Path((module_id, assignment_id)) : Path<(i64, i64)>, Json(payload): Json<PartialConfigUpdate>) -> impl IntoResponse {
    let db = connect.await;

    let assignnment = match AssignmentEntity::find()
        .filter(AssignmentColumn::ModuleId.eq(assignment_id))
        .filter(AssignmentColumn::Id.eq(module_id))
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

    let mut existing_config: serde_json::Map<String, Value> = match assignment.config {
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

        let mut existing_config: serde_json::Map<String, Value> = match assignment.config {
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
    let mut assignment_model: ActiveModel = assignment.into();
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