use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use db::models::{plagiarism_case::{Entity as PlagiarismEntity, Column as PlagiarismColumn}};
use sea_orm::{TransactionTrait, EntityTrait, QueryFilter, ColumnTrait};
use serde::Deserialize;
use util::state::AppState;
use crate::response::ApiResponse;

pub async fn delete_plagiarism_case(
    State(app_state): State<AppState>,
    Path((_, _, case_id)): Path<(i64, i64, i64)>,
) -> impl IntoResponse {
    match PlagiarismEntity::delete_by_id(case_id)
        .exec(app_state.db())
        .await
    {
        Ok(result) => {
            if result.rows_affected == 0 {
                return (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::<()>::error("Plagiarism case not found")),
                );
            }
            
            (
                StatusCode::OK,
                Json(ApiResponse::success_without_data("Plagiarism case deleted successfully")),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!(
                "Failed to delete plagiarism case: {}",
                e
            ))),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct BulkDeletePayload {
    case_ids: Vec<i64>,
}

pub async fn bulk_delete_plagiarism_cases(
    State(app_state): State<AppState>,
    Path((_, assignment_id)): Path<(i64, i64)>,
    Json(payload): Json<BulkDeletePayload>,
) -> impl IntoResponse {
    if payload.case_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("case_ids cannot be empty")),
        );
    }

    let txn = match app_state.db().begin().await {
        Ok(txn) => txn,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!(
                    "Failed to start transaction: {}",
                    e
                ))),
            )
        }
    };

    let existing_cases = match PlagiarismEntity::find()
        .filter(PlagiarismColumn::Id.is_in(payload.case_ids.clone()))
        .filter(PlagiarismColumn::AssignmentId.eq(assignment_id))
        .all(&txn)
        .await
    {
        Ok(cases) => cases,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!(
                    "Database error: {}",
                    e
                ))),
            )
        }
    };

    if existing_cases.len() != payload.case_ids.len() {
        let existing_ids: Vec<i64> = existing_cases.iter().map(|c| c.id).collect();
        let missing_ids: Vec<i64> = payload.case_ids
            .iter()
            .filter(|id| !existing_ids.contains(id))
            .map(|&id| id as i64)
            .collect();

        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(format!(
                "Some plagiarism cases not found or not in assignment: {:?}",
                missing_ids
            ))),
        );
    }

    let delete_result = match PlagiarismEntity::delete_many()
        .filter(PlagiarismColumn::Id.is_in(payload.case_ids))
        .exec(&txn)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!(
                    "Failed to delete plagiarism cases: {}",
                    e
                ))),
            )
        }
    };

    if let Err(e) = txn.commit().await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!(
                "Transaction commit failed: {}",
                e
            ))),
        );
    }

    (
        StatusCode::OK,
        Json(ApiResponse::success_without_data(&format!("{} plagiarism case{} deleted successfully", delete_result.rows_affected, if delete_result.rows_affected == 1 { "" } else { "s" }))),
    )
}