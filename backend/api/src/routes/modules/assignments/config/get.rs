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
