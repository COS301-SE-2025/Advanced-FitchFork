//! Task Details Endpoint
//!
//! This module provides the endpoint handler for retrieving detailed information about a specific assignment task within a module, including its subsections and associated memo outputs. It interacts with the database to validate module, assignment, and task existence, loads the mark allocator configuration, and parses memo output files to provide detailed feedback for each subsection of the task.

use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use serde_json::Value;
use sea_orm::{EntityTrait, QueryFilter, QueryOrder, ColumnTrait};
use std::{env, fs, path::PathBuf};

use crate::response::ApiResponse;
use crate::routes::modules::assignments::tasks::common::TaskResponse;
use db::connect;
use db::models::{
    assignment,
    assignment::{Column as AssignmentColumn, Entity as AssignmentEntity},
    assignment_task::{Column, Entity},
    module,
};
use util::mark_allocator::mark_allocator::load_allocator;

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

/// GET /api/modules/:module_id/assignments/:assignment_id/tasks/:task_id
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
///         "mark_value": 10,
///         "memo_output": "Code compiles successfully without errors."
///       },
///       {
///         "name": "Output",
///         "mark_value": 15,
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

    let task = match Entity::find_by_id(task_id).one(&db).await {
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

    let allocator_json: Option<Value> = load_allocator(module_id, assignment_id).await.ok();
    let task_key = format!("task{}", task.task_number);

    let (_task_name, _task_value, subsections) = if let Some(tasks) = allocator_json
        .as_ref()
        .and_then(|v| v.get("tasks"))
        .and_then(|t| t.as_array())
    {
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


    let mut subsection_outputs = parsed_outputs;
    subsection_outputs.resize(subsections.len(), None);

    let detailed_subsections: Vec<SubsectionDetail> = subsections
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            let obj = match s.as_object() {
                Some(obj) => obj,
                None => {
                    eprintln!("Subsection {} is not a JSON object: {:?}", i, s);
                    return None;
                }
            };

            let name = obj.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("<unnamed>")
                .to_string();

            let mark_value = obj.get("value")
                .and_then(|v| v.as_i64())
                .unwrap_or_else(|| {
                    eprintln!("Missing or invalid value for subsection '{}'", name);
                    0
                });

            let memo_output = subsection_outputs.get(i).cloned().flatten();

            Some(SubsectionDetail {
                name,
                mark_value,
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

/// GET /api/modules/:module_id/assignments/:assignment_id/tasks
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
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let db = connect().await;

    let assignment_exists = AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id as i32))
        .filter(AssignmentColumn::ModuleId.eq(module_id as i32))
        .one(&db)
        .await;

    match assignment_exists {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<Vec<TaskResponse>>::error(
                    "Assignment or module not found",
                )),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<TaskResponse>>::error("Database error")),
            )
                .into_response();
        }
    }

    match Entity::find()
        .filter(Column::AssignmentId.eq(assignment_id))
        .order_by_asc(Column::TaskNumber)
        .all(&db)
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
                    created_at: task.created_at,
                    updated_at: task.updated_at,
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
            Json(ApiResponse::<Vec<TaskResponse>>::error("Failed to retrieve tasks")),
        )
            .into_response(),
    }
}