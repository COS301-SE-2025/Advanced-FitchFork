//! Task Details Endpoint
//!
//! This module provides the endpoint handler for retrieving detailed information about a specific assignment task within a module, including its subsections and associated memo outputs. It interacts with the database to validate module, assignment, and task existence, loads the mark allocator configuration, and parses memo output files to provide detailed feedback for each subsection of the task.

use crate::response::ApiResponse;
use crate::routes::modules::assignments::tasks::common::TaskResponse;
use axum::{
    Json,
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
};
use db::models::assignment_task::{Column, Entity};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::Serialize;
use serde_json::Value;
use std::{env, fs, path::PathBuf};
use util::execution_config::ExecutionConfig;
use util::mark_allocator::mark_allocator::load_allocator;

/// Represents the details of a subsection within a task, including its name, mark value, and optional memo output.
#[derive(Serialize)]
pub struct SubsectionDetail {
    /// The name of the subsection.
    pub name: String,
    /// The value value assigned to this subsection.
    pub value: i64,
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

/// GET /api/modules/{module_id}/assignments/{assignment_id}/tasks/{task_id}
///
/// Retrieve detailed information about a specific task within an assignment. Only accessible by lecturers or admins assigned to the module.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment containing the task
/// - `task_id` (i64): The ID of the task to retrieve details for
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "message": "Task details retrieved successfully",
///   "data": {
///     "id": 123,
///     "task_id": 123,
///     "command": "java -cp . Main",
///     "created_at": "2024-01-01T00:00:00Z",
///     "updated_at": "2024-01-01T00:00:00Z",
///     "subsections": [
///       {
///         "name": "Compilation",
///         "value": 10,
///         "memo_output": "Code compiles successfully without errors."
///       },
///       {
///         "name": "Output",
///         "value": 15,
///         "memo_output": "Program produces correct output for all test cases."
///       }
///     ]
///   }
/// }
/// ```
///
/// - `404 Not Found`
/// ```json
/// {
///   "success": false,
///   "message": "Module not found" // or "Assignment not found" or "Task not found" or "Assignment does not belong to this module" or "Task does not belong to this assignment" or "Task not found in allocator"
/// }
/// ```
///
/// - `500 Internal Server Error`
/// ```json
/// {
///   "success": false,
///   "message": "Database error retrieving module" // or "Database error retrieving assignment" or "Database error retrieving task" or "Failed to load mark allocator"
/// }
/// ```
///
pub async fn get_task_details(
    Path((module_id, assignment_id, task_id)): Path<(i64, i64, i64)>,
) -> impl IntoResponse {
    let db = db::get_connection().await;

    let task = Entity::find_by_id(task_id)
        .one(db)
        .await
        .unwrap()
        .unwrap();

    let base_path =
        env::var("ASSIGNMENT_STORAGE_ROOT").unwrap_or_else(|_| "data/assignment_files".into());
    let memo_path = PathBuf::from(&base_path)
        .join(format!("module_{}", module_id))
        .join(format!("assignment_{}", assignment_id))
        .join("memo_output")
        .join(format!("task_{}.txt", task.task_number));

    let memo_content = fs::read_to_string(&memo_path).ok().or_else(|| {
        fs::read_dir(memo_path.parent()?)
            .ok()?
            .filter_map(Result::ok)
            .find_map(|entry| {
                if entry.path().extension().map_or(false, |ext| ext == "txt") {
                    fs::read_to_string(entry.path()).ok()
                } else {
                    None
                }
            })
    });

    // Load the ExecutionConfig to get the custom delimiter
    let separator = match ExecutionConfig::get_execution_config(module_id, assignment_id) {
        Ok(config) => config.marking.deliminator,
        Err(_) => "&-=-&".to_string(), // fallback if config file missing or unreadable
    };

    // --- keep memo parsing logic unchanged ---
    let outputs: Vec<Option<String>> = if let Some(ref memo) = memo_content {
        let parts: Vec<&str> = memo.split(&separator).collect();
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

    // --- UPDATED: read allocator in new keyed shape (with fallback to legacy flat shape) ---
    // New shape example:
    // {
    //   "tasks": [
    //     { "task1": { "name": "...", "task_number": 1, "value": 9, "subsections": [ ... ] } },
    //     { "task2": { ... } }
    //   ],
    //   "total_value": 27
    // }
    //
    // Legacy fallback (still supported):
    // {
    //   "tasks": [
    //     { "task_number": 1, "value": 9, "subsections": [ ... ] },
    //     ...
    //   ],
    //   "total_value": 27
    // }
    let allocator_json: Option<Value> = load_allocator(module_id, assignment_id).await.ok();

    let ( _task_value, subsections_arr ): (i64, Vec<Value>) = if let Some(tasks_arr) = allocator_json
        .as_ref()
        .and_then(|v| v.get("tasks"))
        .and_then(|t| t.as_array())
    {
        // Try new keyed shape first
        let desired_key = format!("task{}", task.task_number);
        let mut found: Option<(i64, Vec<Value>)> = None;

        for entry in tasks_arr {
            if let Some(entry_obj) = entry.as_object() {
                if let Some(inner) = entry_obj.get(&desired_key) {
                    if let Some(inner_obj) = inner.as_object() {
                        let task_value = inner_obj
                            .get("value")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        let subsections = inner_obj
                            .get("subsections")
                            .and_then(|v| v.as_array())
                            .cloned()
                            .unwrap_or_default();
                        found = Some((task_value, subsections));
                        break;
                    }
                }
            }
        }

        // If not found in keyed shape, fall back to legacy flat shape
        if let Some(hit) = found {
            hit
        } else {
            tasks_arr
                .iter()
                .find_map(|entry| {
                    let obj = entry.as_object()?;
                    let tn = obj.get("task_number")?.as_i64()?;
                    if tn == task.task_number as i64 {
                        let val = obj.get("value").and_then(|v| v.as_i64()).unwrap_or(0);
                        let subs = obj
                            .get("subsections")
                            .and_then(|v| v.as_array())
                            .cloned()
                            .unwrap_or_default();
                        Some((val, subs))
                    } else {
                        None
                    }
                })
                .unwrap_or((0, vec![]))
        }
    } else {
        (0, vec![])
    };

    // Align memo outputs vector length with number of subsections
    let mut subsection_outputs = parsed_outputs;
    subsection_outputs.resize(subsections_arr.len(), None);

    // Build response subsections from subsections (name + value) and attach memo_output
    let detailed_subsections: Vec<SubsectionDetail> = subsections_arr
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let obj = match c.as_object() {
                Some(o) => o,
                None => {
                    eprintln!("Subsection {} is not a JSON object: {:?}", i, c);
                    return None;
                }
            };

            let name = obj
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("<unnamed>")
                .to_string();

            let value = obj
                .get("value")
                .and_then(|v| v.as_i64())
                .unwrap_or_else(|| {
                    eprintln!("Missing or invalid value for subsection '{}'", name);
                    0
                });

            let memo_output = subsection_outputs.get(i).cloned().flatten();

            Some(SubsectionDetail {
                name,
                value,
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
        Json(ApiResponse::success(
            resp,
            "Task details retrieved successfully",
        )),
    )
        .into_response()
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/tasks
///
/// Retrieve all tasks for a specific assignment. Only accessible by lecturers or admins assigned to the module.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to list tasks for
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "message": "Tasks retrieved successfully",
///   "data": [
///     {
///       "id": 123,
///       "task_number": 1,
///       "command": "java -cp . Main",
///       "created_at": "2024-01-01T00:00:00Z",
///       "updated_at": "2024-01-01T00:00:00Z"
///     },
///     {
///       "id": 124,
///       "task_number": 2,
///       "command": "python main.py",
///       "created_at": "2024-01-01T00:00:00Z",
///       "updated_at": "2024-01-01T00:00:00Z"
///     }
///   ]
/// }
/// ```
///
/// - `404 Not Found`
/// ```json
/// {
///   "success": false,
///   "message": "Assignment or module not found"
/// }
/// ```
///
/// - `500 Internal Server Error`
/// ```json
/// {
///   "success": false,
///   "message": "Database error" // or "Failed to retrieve tasks"
/// }
/// ```
///
pub async fn list_tasks(
    Path((_, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let db = db::get_connection().await;

    match Entity::find()
        .filter(Column::AssignmentId.eq(assignment_id))
        .order_by_asc(Column::TaskNumber)
        .all(db)
        .await
    {
        Ok(tasks) => {
            let data = tasks
                .into_iter()
                .map(|task| TaskResponse {
                    id: task.id,
                    task_number: task.task_number,
                    name: task.name,
                    command: task.command,
                    created_at: task.created_at.to_rfc3339(),
                    updated_at: task.updated_at.to_rfc3339(),
                })
                .collect::<Vec<_>>();

            (
                StatusCode::OK,
                Json(ApiResponse::success(data, "Tasks retrieved successfully")),
            )
                .into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<Vec<TaskResponse>>::error(
                "Failed to retrieve tasks",
            )),
        )
            .into_response(),
    }
}
