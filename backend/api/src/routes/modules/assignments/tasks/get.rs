//! Task Details Endpoint
//!
//! This module provides the endpoint handler for retrieving detailed information about a specific assignment task within a module, including its subsections and associated memo outputs. It interacts with the database to validate module, assignment, and task existence, loads the mark allocator configuration, and parses memo output files to provide detailed feedback for each subsection of the task.

use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use db::connect;
use db::models::{assignment_task, assignment, module};
use util::mark_allocator::mark_allocator::load_allocator;
use crate::response::ApiResponse;
use serde::Serialize;
use std::env;
use std::fs;
use std::path::PathBuf;
use sea_orm::EntityTrait;
use chrono::DateTime;

/// Represents the details of a subsection within a task, including its name, mark value, and optional memo output.
#[derive(Serialize)]
pub struct SubsectionDetail {
    /// The name of the subsection.
    pub name: String,
    /// The mark value assigned to this subsection.
    pub mark_value: i64,
    /// The memo output content for this subsection, if available.
    pub memo_output: Option<String>,
}

/// The response structure for detailed information about a task, including its subsections.
#[derive(Serialize)]
pub struct TaskDetailResponse {
    /// The unique database ID of the task.
    pub id: i64,
    /// The task's ID (may be the same as `id`).
    pub task_id: i64,
    /// The command associated with this task.
    pub command: String,
    /// The creation timestamp of the task (RFC3339 format).
    pub created_at: String,
    /// The last update timestamp of the task (RFC3339 format).
    pub updated_at: String,
    /// The list of subsections for this task, with details and memo outputs.
    pub subsections: Vec<SubsectionDetail>,
}

/// A minimal response structure for a task, used for listing or summary purposes.
#[derive(Debug, Serialize)]
pub struct TaskResponse {
    /// The unique database ID of the task.
    id: i64,
    /// The task's number (as used in the allocator and memo files).
    task_number: i64,
    /// The command associated with this task.
    command: String,
    /// The creation timestamp of the task.
    created_at: DateTime<chrono::Utc>,
    /// The last update timestamp of the task.
    updated_at: DateTime<chrono::Utc>,
}

/// Retrieves detailed information about a specific task within an assignment and module.
///
/// This handler performs the following steps:
/// 1. Validates the existence of the module, assignment, and task in the database.
/// 2. Ensures the assignment belongs to the module, and the task belongs to the assignment.
/// 3. Loads the mark allocator JSON for the assignment to get task and subsection structure.
/// 4. Attempts to read the memo output file for the task, splitting it into subsection outputs.
/// 5. Constructs a detailed response including subsection names, mark values, and memo outputs.
///
/// # Path Parameters
/// - `module_id`: The ID of the module.
/// - `assignment_id`: The ID of the assignment.
/// - `task_id`: The ID of the task.
///
/// # Returns
/// - `200 OK` with detailed task information if found.
/// - `404 Not Found` if the module, assignment, or task does not exist or does not belong.
/// - `500 Internal Server Error` for database or file system errors.
///
/// The response includes subsection details and memo outputs, if available.
pub async fn get_task_details(
    Path((module_id, assignment_id, task_id)): Path<(i64, i64, i64)>,
) -> impl IntoResponse {
    let db = connect().await;
    let module_exists = match module::Entity::find_by_id(module_id).one(&db).await {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error retrieving module")),
            ).into_response();
        }
    };

    if !module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        ).into_response();
    }

    let assignment_model = match assignment::Entity::find_by_id(assignment_id).one(&db).await {
        Ok(Some(a)) => a,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Assignment not found")),
            ).into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error retrieving assignment")),
            ).into_response();
        }
    };

    if assignment_model.module_id != module_id {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Assignment does not belong to this module")),
        ).into_response();
    }

    let task = match assignment_task::Entity::find_by_id(task_id)
        .one(&db)
        .await
    {
        Ok(Some(t)) => t,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Task not found")),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error retrieving task")),
            )
                .into_response();
        }
    };

    if task.assignment_id != assignment_id {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Task does not belong to this assignment")),
        )
            .into_response();
    }

    let allocator_json = match load_allocator(module_id, assignment_id).await {
        Ok(val) => val,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Failed to load mark allocator")),
            )
                .into_response();
        }
    };

    let tasks = allocator_json.get("tasks").and_then(|t| t.as_array());
    let task_alloc = match tasks.and_then(|arr| {
        arr.iter().find(|obj| {
            obj.as_object()
                .and_then(|o| o.values().next())
                .and_then(|v| v.get("name"))
                .and_then(|name| name.as_str())
                .and_then(|name| {
                    if let Some(num_str) = name.strip_prefix("Task ") {
                        num_str.parse::<i64>().ok()
                    } else {
                        None
                    }
                }) == Some(task.task_number)
        })
    }) {
        Some(obj) => obj,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Task not found in allocator")),
            )
                .into_response();
        }
    };
    let task_obj = task_alloc.as_object().unwrap();
    let (_task_key, task_val) = task_obj.iter().next().unwrap();
    let empty_vec = Vec::new();
    let subsections = task_val.get("subsections").and_then(|s| s.as_array()).unwrap_or(&empty_vec);

    let base_path = env::var("ASSIGNMENT_STORAGE_ROOT").unwrap_or_else(|_| "data/assignment_files".into());
    let memo_path = PathBuf::from(&base_path)
        .join(format!("module_{}", module_id))
        .join(format!("assignment_{}", assignment_id))
        .join("memo_output")
        .join(format!("task_{}.txt", task.task_number));
    let memo_content = fs::read_to_string(&memo_path).ok();

    let memo_content = memo_content.or_else(|| {
        let dir = memo_path.parent()?;
        let entry = fs::read_dir(dir).ok()?.filter_map(|e| e.ok()).find(|e| e.path().is_file() && e.path().extension().map(|x| x == "txt").unwrap_or(false));
        entry.and_then(|e| fs::read_to_string(e.path()).ok())
    });

    let mut subsection_outputs: Vec<Option<String>> = vec![None; subsections.len()];
    if let Some(ref memo) = memo_content {

        let sep = "&-=-&";
        let mut current_idx = 0;
        let mut buf = String::new();
        for line in memo.lines() {
            if line.contains(sep) {
                if current_idx < subsection_outputs.len() {
                    if !buf.trim().is_empty() {
                        subsection_outputs[current_idx] = Some(buf.trim().to_string());
                    }
                    buf.clear();
                    current_idx += 1;
                }
            } else {
                buf.push_str(line);
                buf.push('\n');
            }
        }
        if current_idx < subsection_outputs.len() && !buf.trim().is_empty() {
            subsection_outputs[current_idx] = Some(buf.trim().to_string());
        }
    }

    let subsections: Vec<SubsectionDetail> = subsections
        .iter()
        .enumerate()
        .map(|(i, s)| SubsectionDetail {
            name: s.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string(),
            mark_value: s.get("value").and_then(|v| v.as_i64()).unwrap_or(0),
            memo_output: subsection_outputs.get(i).and_then(|o| o.clone()),
        })
        .collect();

    let resp = TaskDetailResponse {
        id: task.id,
        task_id: task.id,
        command: task.command,
        created_at: task.created_at.to_rfc3339(),
        updated_at: task.updated_at.to_rfc3339(),
        subsections,
    };
    
    (
        StatusCode::OK,
        Json(ApiResponse::success(resp, "Task details retrieved successfully")),
    )
        .into_response()
}