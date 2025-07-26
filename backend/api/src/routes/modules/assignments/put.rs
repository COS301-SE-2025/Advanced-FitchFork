use axum::{extract::{State, Path}, http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;
use crate::response::ApiResponse;
use db::models::assignment::{self, AssignmentType, Status};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, IntoActiveModel, DbErr};
use super::common::{AssignmentRequest, AssignmentResponse, BulkUpdateRequest, BulkUpdateResult};

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
    State(db): State<DatabaseConnection>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(req): Json<AssignmentRequest>,
) -> impl IntoResponse {
    let available_from = match DateTime::parse_from_rfc3339(&req.available_from)
        .map(|dt| dt.with_timezone(&Utc))
    {
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

    let due_date = match DateTime::parse_from_rfc3339(&req.due_date)
        .map(|dt| dt.with_timezone(&Utc))
    {
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

    match assignment::Model::edit(
        &db,
        assignment_id,
        module_id,
        &req.name,
        req.description.as_deref(),
        assignment_type,
        available_from,
        due_date,
    )
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
            Json(ApiResponse::<AssignmentResponse>::error("Assignment not found")),
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
    State(db): State<DatabaseConnection>,
    Path(module_id): Path<i64>,
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
        let res = assignment::Entity::find()
            .filter(assignment::Column::Id.eq(*id))
            .filter(assignment::Column::ModuleId.eq(module_id))
            .one(&db)
            .await;

        match res {
            Ok(Some(model)) => {
                let mut active = model.into_active_model();

                if let Some(available_from) = &req.available_from {
                    if let Ok(dt) = DateTime::parse_from_rfc3339(available_from) {
                        active.available_from = Set(dt.with_timezone(&Utc));
                    }
                }

                if let Some(due_date) = &req.due_date {
                    if let Ok(dt) = DateTime::parse_from_rfc3339(due_date) {
                        active.due_date = Set(dt.with_timezone(&Utc));
                    }
                }

                active.updated_at = Set(Utc::now());

                if active.update(&db).await.is_ok() {
                    updated += 1;
                } else {
                    failed.push(crate::routes::modules::assignments::common::FailedUpdate {
                        id: *id,
                        error: "Failed to save updated assignment".into(),
                    });
                }
            }
            Ok(None) => failed.push(crate::routes::modules::assignments::common::FailedUpdate {
                id: *id,
                error: "Assignment not found".into(),
            }),
            Err(e) => failed.push(crate::routes::modules::assignments::common::FailedUpdate {
                id: *id,
                error: e.to_string(),
            }),
        }
    }

    let result = BulkUpdateResult { updated, failed };

    let message = format!("Updated {}/{} assignments", updated, req.assignment_ids.len());

    (
        StatusCode::OK,
        Json(ApiResponse::success(result, message)),
    )
}

/// PUT /api/modules/:module_id/assignments/:assignment_id/open
///
/// Transition an assignment to `Open`
///
/// Only works if current status is `Ready`, `Closed`, or `Archived`.
pub async fn open_assignment(
    State(db): State<DatabaseConnection>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let assignment = assignment::Entity::find()
        .filter(assignment::Column::Id.eq(assignment_id))
        .filter(assignment::Column::ModuleId.eq(module_id))
        .one(&db)
        .await;

    match assignment {
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

            let mut active = model.into_active_model();
            active.status = Set(Status::Open);
            active.updated_at = Set(Utc::now());

            if active.update(&db).await.is_ok() {
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
            Json(ApiResponse::<()>::error(&format!(
                "Database error: {}",
                e
            ))),
        ),
    }
}

/// PUT /api/modules/:module_id/assignments/:assignment_id/close
///
/// Transition an assignment from `Open` to `Closed`
///
/// Only works if current status is `Open`.
pub async fn close_assignment(
    State(db): State<DatabaseConnection>,
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let assignment = assignment::Entity::find()
        .filter(assignment::Column::Id.eq(assignment_id))
        .filter(assignment::Column::ModuleId.eq(module_id))
        .one(&db)
        .await;

    match assignment {
        Ok(Some(model)) => {
            if model.status != Status::Open {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<()>::error(
                        "Assignment can only be closed if it is in Open state",
                    )),
                );
            }

            let mut active = model.into_active_model();
            active.status = Set(Status::Closed);
            active.updated_at = Set(Utc::now());

            if active.update(&db).await.is_ok() {
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
            Json(ApiResponse::<()>::error(&format!(
                "Database error: {}",
                e
            ))),
        ),
    }
}