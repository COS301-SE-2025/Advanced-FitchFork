//! Assignment management routes.
//!
//! Provides endpoints for managing assignments within a module, including:
//! - Editing assignments (`PUT /api/modules/{module_id}/assignments/{assignment_id}`)
//! - Bulk updating assignments (`PUT /api/modules/{module_id}/assignments/bulk`)
//! - Transitioning assignment status (`Open` / `Close`)
//!
//! Access control:
//! - Only lecturers or admins assigned to a module can edit or bulk update assignments.
//! - Status transitions are controlled and enforced by the system.
//!
//! Notes:
//! - Direct modification of `status` is not allowed through edit/bulk endpoints; status updates are automatic.
//! - All date fields must be in ISO 8601 format (RFC 3339).

use super::common::{AssignmentRequest, AssignmentResponse, BulkUpdateRequest, BulkUpdateResult};
use crate::response::ApiResponse;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use db::models::assignment::{self, AssignmentType, Status};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DbErr, EntityTrait, IntoActiveModel,
    QueryFilter,
};
use util::state::AppState;

/// PUT /api/modules/{module_id}/assignments/{assignment_id}
///
/// Edit an existing assignment in a module. Only accessible by lecturers or admins assigned to the module.
///
/// This endpoint allows updating general details of the assignment but **does not allow editing its status**.
/// Status transitions (e.g., from `setup` to `ready`) are handled automatically based on readiness checks.
///
/// ### Path Parameters
/// - `module_id` (`i64`): The ID of the module containing the assignment.
/// - `assignment_id` (`i64`): The ID of the assignment to edit.
///
/// ### Request Body (JSON)
/// - `name` (`string`, required): The new name of the assignment.
/// - `description` (`string`, optional): The new description of the assignment.
/// - `assignment_type` (`string`, required): The type of assignment. Must be either `"assignment"` or `"practical"`.
/// - `available_from` (`string`, required): The new date/time from which the assignment is available (ISO 8601 format).
/// - `due_date` (`string`, required): The new due date/time for the assignment (ISO 8601 format).
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "message": "Assignment updated successfully",
///   "data": { /* updated assignment details */ }
/// }
/// ```
///
/// - `400 Bad Request`
/// ```json
/// {
///   "success": false,
///   "message": "Invalid available_from datetime format"
/// }
/// ```
///
/// - `404 Not Found`
/// ```json
/// {
///   "success": false,
///   "message": "Assignment not found"
/// }
/// ```
///
/// - `500 Internal Server Error`
/// ```json
/// {
///   "success": false,
///   "message": "Failed to update assignment"
/// }
/// ```
/// ### Notes
/// - The `status` field of the assignment cannot be updated with this endpoint.
/// - Status is managed automatically by the system when all readiness checks pass.
pub async fn edit_assignment(
    Path((_, assignment_id)): Path<(i64, i64)>,
    Json(req): Json<AssignmentRequest>,
) -> impl IntoResponse {
    let available_from =
        match DateTime::parse_from_rfc3339(&req.available_from).map(|dt| dt.with_timezone(&Utc)) {
            Ok(dt) => dt,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<AssignmentResponse>::error(
                        "Invalid available_from datetime format",
                    )),
                );
            }
        };

    let available_from =
        match DateTime::parse_from_rfc3339(&req.available_from).map(|dt| dt.with_timezone(&Utc)) {
            Ok(dt) => dt,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<AssignmentResponse>::error(
                        "Invalid available_from datetime format",
                    )),
                );
            }
        };

    let due_date =
        match DateTime::parse_from_rfc3339(&req.due_date).map(|dt| dt.with_timezone(&Utc)) {
            Ok(dt) => dt,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<AssignmentResponse>::error(
                        "Invalid due_date datetime format",
                    )),
                );
            }
        };

    let assignment_type = match req.assignment_type.parse::<AssignmentType>() {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<AssignmentResponse>::error(
                    "assignment_type must be 'assignment' or 'practical'",
                )),
            );
        }
    };

    match AssignmentService::update(UpdateAssignment {
        id: assignment_id,
        name: Some(req.name),
        description: req.description,
        assignment_type: Some(assignment_type),
        status: None,
        available_from: Some(available_from),
        due_date: Some(due_date),
    })
    .await
    {
        Ok(updated) => {
            let response = AssignmentResponse::from(updated);
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    response,
                    "Assignment updated successfully",
                )),
            )
        }
        Err(DbErr::RecordNotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<AssignmentResponse>::error(
                "Assignment not found",
            )),
        ),
        Err(DbErr::Custom(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<AssignmentResponse>::error(&msg)),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<AssignmentResponse>::error(
                "Failed to update assignment",
            )),
        ),
    }
}

/// PUT /api/modules/:module_id/assignments/bulk
///
/// Bulk update fields on multiple assignments.
/// Only accessible by lecturers or admins assigned to the module.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module.
///
/// ### Request Body (JSON)
/// ```json
/// {
///   "assignment_ids": [123, 124, 125],
///   "available_from": "2024-01-01T00:00:00Z",
///   "due_date": "2024-02-01T00:00:00Z"
/// }
/// ```
///
/// ### Notes
/// - The `status` field of assignments cannot be updated using this endpoint.
/// - Status transitions are handled automatically by the system based on readiness checks.
///
/// ### Responses
///
/// - `200 OK` (all succeeded)
/// ```json
/// {
///   "success": true,
///   "message": "Updated 3/3 assignments",
///   "data": {
///     "updated": 3,
///     "failed": []
///   }
/// }
/// ```
///
/// - `200 OK` (partial failure)
/// ```json
/// {
///   "success": true,
///   "message": "Updated 2/3 assignments",
///   "data": {
///     "updated": 2,
///     "failed": [
///       {
///         "id": 125,
///         "error": "Assignment not found"
///       }
///     ]
///   }
/// }
/// ```
///
/// - `400 Bad Request`
/// ```json
/// {
///   "success": false,
///   "message": "No assignment IDs provided"
/// }
/// ```
pub async fn bulk_update_assignments(
    Path(_): Path<i64>,
    Json(req): Json<BulkUpdateRequest>,
) -> impl IntoResponse {
    if req.assignment_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("No assignment IDs provided")),
        );
    }

    let mut updated = 0;
    let mut failed = Vec::new();

    for id in &req.assignment_ids {
        match AssignmentService::update(UpdateAssignment {
            id: *id,
            name: None,
            description: None,
            assignment_type: None,
            status: None,
            available_from: req
                .available_from
                .as_ref()
                .and_then(|af| DateTime::parse_from_rfc3339(af).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            due_date: req
                .due_date
                .as_ref()
                .and_then(|dd| DateTime::parse_from_rfc3339(dd).ok())
                .map(|dt| dt.with_timezone(&Utc)),
        })
        .await
        {
            Ok(_) => {
                updated += 1;
            }
            Err(e) => {
                failed.push(crate::routes::modules::assignments::common::FailedUpdate {
                    id: *id,
                    error: e.to_string(),
                });
            }
        }
    }

    let result = BulkUpdateResult { updated, failed };

    let message = format!(
        "Updated {}/{} assignments",
        updated,
        req.assignment_ids.len()
    );

    (StatusCode::OK, Json(ApiResponse::success(result, message)))
}

/// PUT /api/modules/:module_id/assignments/:assignment_id/open
///
/// Transition an assignment to `Open`
///
/// Only works if current status is `Ready`, `Closed`, or `Archived`.
pub async fn open_assignment(Path((_, assignment_id)): Path<(i64, i64)>) -> impl IntoResponse {
    match AssignmentService::find_by_id(assignment_id).await {
        Ok(Some(model)) => {
            if !matches!(
                model.status,
                Status::Ready | Status::Closed | Status::Archived
            ) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<()>::error(
                        "Assignment can only be opened if it is in Ready, Closed, or Archived state",
                    )),
                );
            }

            if AssignmentService::update(UpdateAssignment {
                id: assignment_id,
                name: None,
                description: None,
                assignment_type: None,
                status: Some(Status::Open),
                available_from: None,
                due_date: None,
            })
            .await
            .is_ok()
            {
                (
                    StatusCode::OK,
                    Json(ApiResponse::<()>::success(
                        (),
                        "Assignment successfully opened",
                    )),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Failed to update assignment")),
                )
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Assignment not found")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(&format!("Database error: {}", e))),
        ),
    }
}

/// PUT /api/modules/:module_id/assignments/:assignment_id/close
///
/// Transition an assignment from `Open` to `Closed`
///
/// Only works if current status is `Open`.
pub async fn close_assignment(Path((_, assignment_id)): Path<(i64, i64)>) -> impl IntoResponse {
    match AssignmentService::find_by_id(assignment_id).await {
        Ok(Some(model)) => {
            if model.status != Status::Open {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<()>::error(
                        "Assignment can only be closed if it is in Open state",
                    )),
                );
            }

            if AssignmentService::update(UpdateAssignment {
                id: assignment_id,
                name: None,
                description: None,
                assignment_type: None,
                status: Some(Status::Closed),
                available_from: None,
                due_date: None,
            })
            .await
            .is_ok()
            {
                (
                    StatusCode::OK,
                    Json(ApiResponse::<()>::success(
                        (),
                        "Assignment successfully closed",
                    )),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Failed to update assignment")),
                )
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Assignment not found")),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(&format!("Database error: {}", e))),
        ),
    }
}
