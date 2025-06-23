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