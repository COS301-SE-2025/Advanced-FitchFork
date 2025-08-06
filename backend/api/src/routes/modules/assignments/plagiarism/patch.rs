use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use db::models::plagiarism_case::{Entity as PlagiarismEntity, Status, Column as PlagiarismColumn};
use sea_orm::{ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, ActiveModelTrait, Set};
use serde::Serialize;
use util::state::AppState;
use crate::response::ApiResponse;

#[derive(Debug, Serialize)]
pub struct FlaggedCaseResponse {
    id: i64,
    status: String,
    updated_at: DateTime<Utc>,
}

pub async fn patch_plagiarism_flag(
    State(app_state): State<AppState>,
    Path((_, assignment_id, plagiarism_id)): Path<(i64, i64, i64)>,
) -> impl IntoResponse {
    let case = match PlagiarismEntity::find()
        .filter(PlagiarismColumn::Id.eq(plagiarism_id))
        .filter(PlagiarismColumn::AssignmentId.eq(assignment_id))
        .one(app_state.db())
        .await
    {
        Ok(Some(case)) => case,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("Plagiarism case not found")),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!("Database error: {}", e))),
            )
        }
    };

    let mut active_case = case.into_active_model();
    active_case.status = Set(Status::Flagged);
    active_case.updated_at = Set(Utc::now());

    let updated_case = match active_case.update(app_state.db()).await {
        Ok(case) => case,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!("Failed to update plagiarism case: {}", e))),
            )
        }
    };

    let response_data = FlaggedCaseResponse {
        id: updated_case.id as i64,
        status: updated_case.status.to_string(),
        updated_at: updated_case.updated_at,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(response_data, "Plagiarism case flagged")),
    )
}

#[derive(Debug, Serialize)]
pub struct ReviewedCaseResponse {
    id: i64,
    status: String,
    updated_at: DateTime<Utc>,
}

pub async fn patch_plagiarism_review(
    State(app_state): State<AppState>,
    Path((_, assignment_id, plagiarism_id)): Path<(i64, i64, i64)>,
) -> impl IntoResponse {
    let case = match PlagiarismEntity::find()
        .filter(PlagiarismColumn::Id.eq(plagiarism_id))
        .filter(PlagiarismColumn::AssignmentId.eq(assignment_id))
        .one(app_state.db())
        .await
    {
        Ok(Some(case)) => case,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("Plagiarism case not found")),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!("Database error: {}", e))),
            )
        }
    };

    let mut active_case = case.into_active_model();
    active_case.status = Set(Status::Reviewed);
    active_case.updated_at = Set(Utc::now());

    let updated_case = match active_case.update(app_state.db()).await {
        Ok(case) => case,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!("Failed to update plagiarism case: {}", e))),
            )
        }
    };

    let response_data = ReviewedCaseResponse {
        id: updated_case.id as i64,
        status: updated_case.status.to_string(),
        updated_at: updated_case.updated_at,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(response_data, "Plagiarism case marked as reviewed")),
    )
}