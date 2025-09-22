use std::fs;

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use db::models::{
    moss_report::{Column as MossReportColumn, Entity as MossReportEntity},
    plagiarism_case::{Column as PlagiarismColumn, Entity as PlagiarismEntity},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use serde::Deserialize;
use util::{paths::moss_archive_dir, state::AppState};

use crate::response::ApiResponse;
use util::filters::FilterParam;
use services::service::Service;
use services::plagiarism_case::PlagiarismCaseService;

/// DELETE /api/modules/{module_id}/assignments/{assignment_id}/plagiarism/{case_id}
///
/// Permanently deletes a plagiarism case from the system.
/// Accessible only to lecturers and assistant lecturers assigned to the module.
///
/// # Path Parameters
///
/// - `module_id`: The ID of the parent module
/// - `assignment_id`: The ID of the assignment containing the plagiarism case
/// - `case_id`: The ID of the plagiarism case to delete
///
/// # Returns
///
/// Returns an HTTP response indicating the result:
/// - `200 OK` with success message when deletion is successful
/// - `403 FORBIDDEN` if user lacks required permissions
/// - `404 NOT FOUND` if specified plagiarism case doesn't exist
/// - `500 INTERNAL SERVER ERROR` for database errors or deletion failures
///
/// The response body follows a standardized JSON format with a success message.
///
/// # Example Response (200 OK)
///
/// ```json
/// {
///   "success": true,
///   "message": "Plagiarism case deleted successfully"
/// }
/// ```
///
/// # Example Responses
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "message": "Plagiarism case not found"
/// }
/// ```
///
/// - `500 Internal Server Error`  
/// ```json
/// {
///   "success": false,
///   "message": "Failed to delete plagiarism case: [error details]"
/// }
/// ```
///
/// # Notes
///
/// This operation is irreversible and permanently removes the plagiarism case record.
/// Only users with lecturer or assistant lecturer roles assigned to the module can perform this action.
pub async fn delete_plagiarism_case(
    Path((_, _, case_id)): Path<(i64, i64, i64)>,
) -> impl IntoResponse {
    match PlagiarismCaseService::delete_by_id(case_id).await {
        Ok(_) => {
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

/// DELETE /api/modules/{module_id}/assignments/{assignment_id}/plagiarism/bulk
///
/// Deletes multiple plagiarism cases in bulk for a specific assignment.
/// Accessible only to lecturers and assistant lecturers assigned to the module.
///
/// # Path Parameters
///
/// - `module_id`: The ID of the parent module
/// - `assignment_id`: The ID of the assignment containing the plagiarism cases
///
/// # Request Body
///
/// Requires a JSON payload with the following field:
/// - `case_ids`: Array of plagiarism case IDs to delete (must not be empty)
///
/// # Returns
///
/// Returns an HTTP response indicating the result:
/// - `200 OK` with success message and count of deleted cases
/// - `400 BAD REQUEST` for invalid payload or missing cases
/// - `403 FORBIDDEN` if user lacks required permissions
/// - `500 INTERNAL SERVER ERROR` for database errors or deletion failures
///
/// The response body follows a standardized JSON format with a success message.
///
/// # Example Request
///
/// ```json
/// {
///   "case_ids": [12, 13, 17]
/// }
/// ```
///
/// # Example Response (200 OK)
///
/// ```json
/// {
///   "success": true,
///   "message": "3 plagiarism cases deleted successfully"
/// }
/// ```
///
/// # Example Responses
///
/// - `400 Bad Request` (empty case_ids)  
/// ```json
/// {
///   "success": false,
///   "message": "case_ids cannot be empty"
/// }
/// ```
///
/// - `400 Bad Request` (missing cases)  
/// ```json
/// {
///   "success": false,
///   "message": "Some plagiarism cases not found or not in assignment: [17, 25]"
/// }
/// ```
///
/// - `500 Internal Server Error` (transaction failure)  
/// ```json
/// {
///   "success": false,
///   "message": "Transaction commit failed: [error details]"
/// }
/// ```
///
/// # Notes
///
/// - This operation is atomic - either all cases are deleted or none
/// - Returns an error if any specified case doesn't exist or belongs to a different assignment
/// - Only users with lecturer or assistant lecturer roles assigned to the module can perform this action
pub async fn bulk_delete_plagiarism_cases(
    Path((_, assignment_id)): Path<(i64, i64)>,
    Json(payload): Json<BulkDeletePayload>,
) -> impl IntoResponse {
    if payload.case_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error("case_ids cannot be empty")),
        );
    }

    let existing_cases = match PlagiarismCaseService::find_all(
        &vec![
            FilterParam::eq("id", payload.case_ids.clone()),
            FilterParam::eq("assignment_id", assignment_id),
        ],
        &vec![],
        None,
    ).await {
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

    let delete_result = match PlagiarismCaseService::delete(
        &vec![
            FilterParam::eq("id", payload.case_ids),
            FilterParam::eq("assignment_id", assignment_id),
        ],
        &vec![],
    ).await {
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

    (
        StatusCode::OK,
        Json(ApiResponse::success_without_data(&format!("{} plagiarism case{} deleted successfully", delete_result, if delete_result == 1 { "" } else { "s" }))),
    )
}

/// DELETE /api/modules/{module_id}/assignments/{assignment_id}/plagiarism/moss/reports/{report_id}
///
/// Deletes a **single MOSS report** record. If a versioned archive exists for this report,
/// it is also removed from disk.
///
/// # Path
/// - `module_id`
/// - `assignment_id`
/// - `report_id`
///
/// # Responses
/// - 200 OK — report (and its archive, if present) deleted
/// - 404 Not Found — report not found or not in assignment
/// - 500 Internal Server Error — failed to delete DB row or archive folder
pub async fn delete_moss_report(
    State(app_state): State<AppState>,
    Path((module_id, assignment_id, report_id)): Path<(i64, i64, i64)>,
) -> impl IntoResponse {
    // 1) Ensure the report exists and belongs to this assignment
    let report = match MossReportEntity::find_by_id(report_id)
        .filter(MossReportColumn::AssignmentId.eq(assignment_id))
        .one(app_state.db())
        .await
    {
        Ok(Some(m)) => m,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Report not found")),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!(
                    "Failed to query report: {e}"
                ))),
            )
                .into_response();
        }
    };

    // 2) Best-effort: remove archive folder if present (we use report_id as the archive id)
    //    NOTE: this assumes archives are stored under <...>/moss_archives/{report_id}/archive.zip
    let archive_dir = moss_archive_dir(module_id, assignment_id, &report_id.to_string());
    if archive_dir.exists() {
        if let Err(e) = fs::remove_dir_all(&archive_dir) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!(
                    "Failed to delete report archive: {e}"
                ))),
            )
                .into_response();
        }
    }

    // 3) Delete the DB row
    match MossReportEntity::delete_by_id(report.id).exec(app_state.db()).await {
        Ok(res) if res.rows_affected > 0 => (
            StatusCode::OK,
            Json(ApiResponse::<()>::success_without_data("Report deleted successfully")),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Report not found")),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(format!("Failed to delete report: {e}"))),
        )
            .into_response(),
    }
}