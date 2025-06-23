use axum::{extract::{Path}, http::StatusCode, response::IntoResponse, Extension, Json};
use chrono::{DateTime, Utc};
use db::{
    connect,
    models::{
        assignment::{
            Column as AssignmentColumn, Entity as AssignmentEntity,
        }, assignment_submission
    },
};
use sea_orm::{
    ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};
use serde::Serialize;

use crate::{auth::AuthUser, response::ApiResponse};

/// GET /api/modules/:module_id/assignments/:assignment_id/submissions/me
///
/// Get a list of the current user's submissions for a specific assignment.
///
/// ### Responses
/// - `200 OK` with list of submissions
/// - `404 Not Found` (assignment not found)
/// - `500 Internal Server Error` (database error)
///
pub fn is_late(submission: DateTime<Utc>, due_date: DateTime<Utc>) -> bool {
    submission > due_date
}

#[derive(Debug, Serialize)]
pub struct SubmissionResponse {
    pub id: i64,
    pub filename: String,
    pub created_at: String,
    pub is_late: bool,
}

pub async fn get_user_submissions(
    Path((module_id, assignment_id, user_id)): Path<(i64, i64, i64)>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> impl IntoResponse {
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
                Json(ApiResponse::<Vec<SubmissionResponse>>::error(
                    "Assignment not found",
                )),
            )
                .into_response();
        }
        Err(err) => {
            eprintln!("DB error checking assignment: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<SubmissionResponse>>::error(
                    "Database error",
                )),
            )
                .into_response();
        }
    };

    match assignment_submission::Entity::find()
        .filter(assignment_submission::Column::AssignmentId.eq(assignment_id as i32))
        .filter(assignment_submission::Column::UserId.eq(user_id))
        .order_by_desc(assignment_submission::Column::CreatedAt)
        .all(&db)
        .await
    {
        Ok(submissions) => {
            let response: Vec<SubmissionResponse> = submissions
                .into_iter()
                .map(|s| SubmissionResponse {
                    id: s.id,
                    filename: s.filename,
                    created_at: s.created_at.to_rfc3339(),
                    is_late: is_late(s.created_at, assignment.due_date),
                })
                .collect();

            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "Submissions retrieved successfully",
                )),
            )
                .into_response()
        }
        Err(err) => {
            eprintln!("DB error fetching submissions: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<SubmissionResponse>>::error(
                    "Failed to retrieve submissions",
                )),
            )
                .into_response()
        }
    }
}


