//! Task Details Endpoint
//!
//! This module provides the endpoint handler for retrieving detailed information about a specific assignment task within a module, including its subsections and associated memo outputs. It interacts with the database to validate module, assignment, and task existence, loads the mark allocator configuration, and parses memo output files to provide detailed feedback for each subsection of the task.

use crate::response::ApiResponse;
use crate::routes::modules::assignments::tasks::common::TaskResponse;
use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use db::models::assignment::Entity as AssignmentEntity;
use db::models::assignment_memo_output::{Column as MemoCol, Entity as MemoEntity};
use db::models::assignment_overwrite_file::{
    Column as OverwriteFileColumn, Entity as OverwriteFileEntity,
};
use db::models::assignment_task::{Column, Entity};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use util::paths::memo_output_dir;
use util::{execution_config::ExecutionConfig, state::AppState};
use util::{mark_allocator::mark_allocator::load_allocator, paths::storage_root};

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
    pub code_coverage: bool,
    /// The creation timestamp of the task (RFC3339 format).
    pub created_at: String,
    /// The last update timestamp of the task (RFC3339 format).
    pub updated_at: String,
    pub has_overwrite_files: bool,
    /// The list of subsections for this task, with details and memo outputs.
    pub subsections: Vec<SubsectionDetail>,
}

/// GET /api/modules/{module_id}/assignments/{assignment_id}/tasks/{task_id}
///
/// Retrieve detailed information about a specific task within an assignment. Only accessible
/// by lecturers or admins assigned to the module.
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
///     "name": "Compilation",
///     "command": "java -cp . Main",
///     "code_coverage": false,
///     "has_overwrite_files": false,
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
///   "message": "Module not found"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Assignment not found"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Task not found"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Assignment does not belong to this module"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Task does not belong to this assignment"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Task not found in allocator"
/// }
/// ```
///
/// - `500 Internal Server Error`
/// ```json
/// {
///   "success": false,
///   "message": "Database error retrieving module"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Database error retrieving assignment"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Database error retrieving task"
/// }
/// ```
/// or
/// ```json
/// {
///   "success": false,
///   "message": "Failed to load mark allocator"
/// }
/// ```
///
/// ### Notes
/// - `code_coverage: true` marks this task as a **coverage-type** task (special handling by the evaluator).
/// - `subsections[*].memo_output` is parsed from the memo output file using the configured delimiter.
pub async fn get_task_details(
    Path((module_id, assignment_id, task_id)): Path<(i64, i64, i64)>,
) -> impl IntoResponse {
    let db = app_state.db();

    // Load task
    let task = match Entity::find_by_id(task_id).one(db).await {
        Ok(Some(task)) => task,
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

    // Validate assignment exists and belongs to module
    let assignment = match AssignmentEntity::find_by_id(assignment_id).one(db).await {
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
                Json(ApiResponse::<()>::error(
                    "Database error retrieving assignment",
                )),
            )
                .into_response();
        }
    };
    if assignment.module_id != module_id {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(
                "Assignment does not belong to this module",
            )),
        )
            .into_response();
    }
    if task.assignment_id != assignment_id {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(
                "Task does not belong to this assignment",
            )),
        )
            .into_response();
    }

    // ── Memo output: prefer DB mapping; fall back to scanning memo_output_dir ─────────────
    let memo_content: Option<String> = match MemoEntity::find()
        .filter(MemoCol::AssignmentId.eq(assignment_id))
        .filter(MemoCol::TaskId.eq(task.id))
        .one(db)
        .await
    {
        Ok(Some(mo)) => {
            // The model stores a relative path from its storage root
            let full = storage_root().join(&mo.path);
            fs::read_to_string(full).ok()
        }
        _ => {
            // Fallback: look for a txt file in the central memo_output dir
            let dir = memo_output_dir(module_id, assignment_id);
            // First try conventional "task_{n}.txt"
            let conventional = dir.join(format!("task_{}.txt", task.task_number));
            if let Ok(s) = fs::read_to_string(&conventional) {
                Some(s)
            } else {
                // Else, pick the first *.txt file (stable-ish order by name)
                let mut picked: Option<String> = None;
                if let Ok(rd) = fs::read_dir(&dir) {
                    let mut txts: Vec<_> = rd
                        .flatten()
                        .map(|e| e.path())
                        .filter(|p| p.is_file() && p.extension().map_or(false, |e| e == "txt"))
                        .collect();
                    txts.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
                    for p in txts {
                        if let Ok(s) = fs::read_to_string(&p) {
                            picked = Some(s);
                            break;
                        }
                    }
                }
                picked
            }
        }
    };

    // Load ExecutionConfig delimiter
    let separator = match ExecutionConfig::get_execution_config(module_id, assignment_id) {
        Ok(cfg) => cfg.marking.deliminator,
        Err(_) => "&-=-&".to_string(), // sensible fallback
    };

    // Parse memo outputs into Optional lines per subsection (skip the leading chunk)
    let parts: Vec<Option<String>> = if let Some(ref memo) = memo_content {
        memo.split(&separator)
            .map(|s| {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            })
            .collect()
    } else {
        Vec::new()
    };
    let mut subsection_outputs: Vec<Option<String>> = parts.into_iter().skip(1).collect();

    // ── Load allocator and extract this task’s subsections (new keyed or legacy flat) ─────
    let allocator_json = load_allocator(module_id, assignment_id).await.ok();
    let (_, subsections_arr): (i64, Vec<Value>) = if let Some(tasks_arr) = allocator_json
        .as_ref()
        .and_then(|v| v.get("tasks"))
        .and_then(|t| t.as_array())
    {
        // Try new keyed shape: { "tasks": [ { "task{n}": { ... } }, ... ] }
        let desired_key = format!("task{}", task.task_number);
        if let Some((val, subs)) = tasks_arr.iter().find_map(|entry| {
            entry.as_object().and_then(|obj| {
                obj.get(&desired_key).and_then(|inner| {
                    inner.as_object().map(|o| {
                        let val = o.get("value").and_then(|v| v.as_i64()).unwrap_or(0);
                        let subs = o
                            .get("subsections")
                            .and_then(|v| v.as_array())
                            .cloned()
                            .unwrap_or_default();
                        (val, subs)
                    })
                })
            })
        }) {
            (val, subs)
        } else {
            // Legacy flat: { "tasks": [ { "task_number": n, "value": ..., "subsections": [...] }, ...] }
            tasks_arr
                .iter()
                .find_map(|entry| {
                    let obj = entry.as_object()?;
                    let tn = obj.get("task_number")?.as_i64()?; // safely extract task_number
                    if tn == task.task_number {
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

    // Align memo outputs to subsections length
    if subsection_outputs.len() < subsections_arr.len() {
        subsection_outputs.resize(subsections_arr.len(), None);
    }

    // Build detailed subsections
    let subsections: Vec<SubsectionDetail> = subsections_arr
        .iter()
        .enumerate()
        .filter_map(|(i, v)| {
            let o = v.as_object()?;
            let name = o
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("<unnamed>")
                .to_string();
            let value = o.get("value").and_then(|x| x.as_i64()).unwrap_or(0);
            let memo_output = subsection_outputs.get(i).cloned().flatten();
            Some(SubsectionDetail {
                name,
                value,
                memo_output,
            })
        })
        .collect();

    // Overwrite files?
    let has_overwrite_files = match OverwriteFileEntity::find()
        .filter(OverwriteFileColumn::AssignmentId.eq(assignment_id))
        .filter(OverwriteFileColumn::TaskId.eq(task.id))
        .count(db)
        .await
    {
        Ok(c) => c > 0,
        Err(e) => {
            eprintln!("DB error counting overwrite files: {:?}", e);
            false
        }
    };

    let resp = TaskDetailResponse {
        id: task.id,
        task_id: task.id,
        name: task.name.clone(),
        command: task.command,
        code_coverage: task.code_coverage,
        created_at: task.created_at.to_rfc3339(),
        updated_at: task.updated_at.to_rfc3339(),
        has_overwrite_files,
        subsections,
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
/// Retrieve all tasks for a specific assignment. Only accessible by lecturers or admins
/// assigned to the module. Results are sorted by `task_number` ascending.
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
///       "name": "Compilation",
///       "command": "java -cp . Main",
///       "code_coverage": false,
///       "created_at": "2024-01-01T00:00:00Z",
///       "updated_at": "2024-01-01T00:00:00Z"
///     },
///     {
///       "id": 124,
///       "task_number": 2,
///       "name": "Coverage run",
///       "command": "cargo llvm-cov --no-report",
///       "code_coverage": true,
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
/// ### Notes
/// - `code_coverage: true` marks a task as a **coverage-type** task.
/// - List is ordered by `task_number` ascending.
pub async fn list_tasks(Path((_, assignment_id)): Path<(i64, i64)>) -> impl IntoResponse {
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
                    code_coverage: task.code_coverage,
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
