use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use db::models::{
    assignment::{Column as AssignmentColumn, Entity as AssignmentEntity},
    plagiarism_case::{Entity as PlagiarismEntity, Status, Column as PlagiarismColumn},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, ActiveModelTrait, IntoActiveModel};
use serde::{Deserialize, Serialize};
use util::state::AppState;
use crate::response::ApiResponse;

#[derive(Deserialize)]
pub struct UpdatePlagiarismCasePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Serialize)]
pub struct PlagiarismCaseResponse {
    id: i64,
    assignment_id: i64,
    submission_id_1: i64,
    submission_id_2: i64,
    description: String,
    status: String,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

pub async fn update_plagiarism_case(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id, plagiarism_id)): Path<(i64, i64, i64)>,
    Json(payload): Json<UpdatePlagiarismCasePayload>,
) -> Result<Json<ApiResponse<PlagiarismCaseResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    if payload.description.is_none() && payload.status.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("At least one field (description or status) must be provided".to_string())),
        ));
    }

    let assignment = AssignmentEntity::find_by_id(assignment_id)
        .filter(AssignmentColumn::ModuleId.eq(module_id))
        .one(app_state.db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
            )
        })?;

    if assignment.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(format!("Assignment {} not found in module {}", assignment_id, module_id))),
        ));
    }

    let case = PlagiarismEntity::find_by_id(plagiarism_id)
        .filter(PlagiarismColumn::AssignmentId.eq(assignment_id))
        .one(app_state.db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Database error: {}", e))),
            )
        })?;

    let mut case = if let Some(case) = case {
        case
    } else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(format!("Plagiarism case {} not found", plagiarism_id))),
        ));
    };

    if let Some(description) = payload.description {
        case.description = description;
    }

    if let Some(status_str) = payload.status {
        let status = match status_str.as_str() {
            "review" => Status::Review,
            "flagged" => Status::Flagged,
            "reviewed" => Status::Reviewed,
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<()>::error("Invalid status value. Must be one of: 'review', 'flagged', 'reviewed'".to_string())),
                ));
            }
        };
        case.status = status;
    }

    case.updated_at = Utc::now();

    let updated_case = case.into_active_model()
        .update(app_state.db())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!("Failed to update plagiarism case: {}", e))),
            )
        })?;

    let response = PlagiarismCaseResponse {
        id: updated_case.id,
        assignment_id: updated_case.assignment_id,
        submission_id_1: updated_case.submission_id_1,
        submission_id_2: updated_case.submission_id_2,
        description: updated_case.description,
        status: updated_case.status.to_string(),
        created_at: updated_case.created_at,
        updated_at: updated_case.updated_at,
    };

    Ok(Json(ApiResponse::success(response, "Plagiarism case updated successfully")))
}