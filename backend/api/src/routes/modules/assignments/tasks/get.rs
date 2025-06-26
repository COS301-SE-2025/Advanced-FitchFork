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
use serde_json::Value;

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
    /// The display name assigned to a task
    pub name: String,  
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

    // Validate module
    let module_exists = match module::Entity::find_by_id(module_id).one(&db).await {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error retrieving module")),
            )
                .into_response();
        }
    };
    if !module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Module not found")),
        )
            .into_response();
    }

    // Validate assignment
    let assignment_model = match assignment::Entity::find_by_id(assignment_id).one(&db).await {
        Ok(Some(a)) => a,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error("Assignment not found")),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Database error retrieving assignment")),
            )
                .into_response();
        }
    };
    if assignment_model.module_id != module_id {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("Assignment does not belong to this module")),
        )
            .into_response();
    }

    // Validate task
    let task = match assignment_task::Entity::find_by_id(task_id).one(&db).await {
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

    let allocator_json: Option<Value> = load_allocator(module_id, assignment_id).await.ok();
    let task_key = format!("task{}", task.task_number);

    let (_task_name, _task_value, subsections) = if let Some(tasks) = allocator_json.as_ref().and_then(|v| v.as_array()) {
        tasks.iter().find_map(|obj| {
            obj.as_object().and_then(|map| {
                let task_obj = map.get(&task_key)?.as_object()?;
                let name = task_obj.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                let value = task_obj.get("value").and_then(|v| v.as_i64()).unwrap_or(0);
                let subsections = task_obj.get("subsections")?.as_array()?.clone();
                Some((name, value, subsections))
            })
        }).unwrap_or_else(|| ("".to_string(), 0, vec![]))
    } else {
        ("".to_string(), 0, vec![])
    };


    let base_path = env::var("ASSIGNMENT_STORAGE_ROOT")
        .unwrap_or_else(|_| "data/assignment_files".into());
    let memo_path = PathBuf::from(&base_path)
        .join(format!("module_{}", module_id))
        .join(format!("assignment_{}", assignment_id))
        .join("memo_output")
        .join(format!("task_{}.txt", task.task_number));

    let memo_content = fs::read_to_string(&memo_path).ok().or_else(|| {
        fs::read_dir(memo_path.parent()?).ok()?.filter_map(Result::ok).find_map(|entry| {
            if entry.path().extension().map_or(false, |ext| ext == "txt") {
                fs::read_to_string(entry.path()).ok()
            } else {
                None
            }
        })
    });

    let outputs: Vec<Option<String>> = if let Some(ref memo) = memo_content {
        let parts: Vec<&str> = memo.split("&-=-&").collect();
        parts
            .into_iter()
            .map(|part| {
                let trimmed = part.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            })
            .collect()
    } else {
        vec![]
    };

    let parsed_outputs: Vec<Option<String>> = outputs.into_iter().skip(1).collect();

    if allocator_json.is_none() || subsections.is_empty() {
        let generated_subsections: Vec<SubsectionDetail> = parsed_outputs
            .iter()
            .enumerate()
            .map(|(i, output)| SubsectionDetail {
                name: format!("Subsection {}", i + 1),
                mark_value: 0,
                memo_output: output.clone(),
            })
            .collect();

        let resp = TaskDetailResponse {
            id: task.id,
            task_id: task.id,
            name: task.name.clone(), 
            command: task.command,
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.updated_at.to_rfc3339(),
            subsections: generated_subsections,
        };

        return (
            StatusCode::OK,
            Json(ApiResponse::success(resp, "Task details retrieved without allocator")),
        )
            .into_response();
    }

    let mut subsection_outputs = parsed_outputs;
    subsection_outputs.resize(subsections.len(), None);

    let detailed_subsections: Vec<SubsectionDetail> = subsections
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            let obj = s.as_object()?;
            let name = obj.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
            let value = obj.get("value").and_then(|v| v.as_i64()).unwrap_or(0);
            let memo_output = subsection_outputs.get(i).cloned().flatten();

            Some(SubsectionDetail {
                name,
                mark_value: value,
                memo_output,
            })
        })
        .collect();

    let resp = TaskDetailResponse {
        id: task.id,
        task_id: task.id,
        name: task.name.clone(), 
        command: task.command,
        created_at: task.created_at.to_rfc3339(),
        updated_at: task.updated_at.to_rfc3339(),
        subsections: detailed_subsections,
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(resp, "Task details retrieved successfully")),
    )
        .into_response()
}
