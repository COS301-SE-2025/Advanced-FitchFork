use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use db::models::{plagiarism_case::Entity as PlagiarismEntity,};
use sea_orm::EntityTrait;
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