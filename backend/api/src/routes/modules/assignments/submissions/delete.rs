use axum::{extract::Path, Json, http::StatusCode, response::IntoResponse};
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, TransactionTrait, DatabaseConnection};
use db::models::assignment_submission::{Entity as SubmissionEntity, Column as SubmissionColumn};
use serde::Deserialize;
use crate::response::ApiResponse;
use std::fs;


#[derive(Debug, Deserialize)]
pub struct DeleteSubmissionsRequest {
    pub submission_ids: Vec<i64>,
}

pub async fn delete_submissions(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(body): Json<DeleteSubmissionsRequest>,
) -> impl IntoResponse {
    if body.submission_ids.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<()>::error("submission_ids must not be empty")),
        )
            .into_response();
    }

    let db = db::connect().await;

    // Confirm assignment exists
    let exists = db::models::assignment::Entity::find()
        .filter(db::models::assignment::Column::Id.eq(assignment_id as i32))
        .filter(db::models::assignment::Column::ModuleId.eq(module_id as i32))
        .one(&db)
        .await;

    match exists {
        Ok(Some(_)) => {}
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
    }

    let submissions = SubmissionEntity::find()
        .filter(SubmissionColumn::Id.is_in(body.submission_ids.clone()))
        .filter(SubmissionColumn::AssignmentId.eq(assignment_id))
        .all(&db)
        .await;

    let Ok(records) = submissions else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("Failed to query submissions")),
        )
            .into_response();
    };

    if records.len() != body.submission_ids.len() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ApiResponse::<()>::error(
                "One or more submission IDs are invalid or do not belong to this assignment",
            )),
        )
            .into_response();
    }

    let t = db.begin().await.unwrap();

    for submission in &records {
       
        if let Err(e) = fs::remove_file(&submission.path) {
            eprintln!("Warning: failed to delete file '{}': {:?}", submission.path, e);
        }

        let result = SubmissionEntity::delete_by_id(submission.id).exec(&t).await;
        if result.is_err() {
            t.rollback().await.unwrap();
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to delete submissions")),
            )
                .into_response();
        }
    }

    t.commit().await.unwrap();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            None::<()>,
            &format!("{} submissions deleted successfully", records.len()),
        )),
    )
        .into_response()
}