use crate::response::ApiResponse;
use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde_json::Value;
use util::mark_allocator::mark_allocator::{SaveError, save_allocator};

/// PUT /api/modules/{module_id}/assignments/{assignment_id}/mark_allocator
///
/// Save the mark allocator JSON configuration for a specific assignment. Accessible to users with
/// Lecturer roles assigned to the module.
///
/// This endpoint saves a mark allocator configuration to the assignment directory. The configuration
/// defines how many **points** (values) are allocated to each task and its subsections. The saved
/// configuration is used by the grading system to calculate final marks.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to save the mark allocator for
///
/// ### Request Body (points-based schema using `value`)
/// A JSON object with:
/// - `tasks`: non-empty array of **single-key** task objects: `{ "taskN": { ... } }`
/// - `total_value`: integer (≥ 0) equal to the sum of all task values
///
/// ```json
/// {
///   "generated_at": "2025-08-17T22:00:00Z",
///   "tasks": [
///     {
///       "task1": {
///         "name": "Task 1",
///         "task_number": 1,
///         "value": 9,
///         "subsections": [
///           { "name": "Correctness", "value": 5 },
///           { "name": "Style",       "value": 4 }
///         ]
///       }
///     },
///     {
///       "task2": {
///         "name": "Task 2",
///         "task_number": 2,
///         "value": 6,
///         "subsections": [
///           { "name": "Docs", "value": 2 },
///           { "name": "Tests","value": 4 }
///         ]
///       }
///     }
///   ],
///   "total_value": 15
/// }
/// ```
///
/// #### Field semantics
/// - `tasks` (required, non-empty array): Each element must be an object with **exactly one** key `"taskN"`.
///   The value of that key is a task object with:
///   - `name` (required, non-empty string)
///   - `task_number` (required, positive integer). Must match the number N in `"taskN"`.
///   - `value` (required, integer ≥ 0): Total points for the task
///   - `subsections` (required, array; may be empty):
///     - Each item: `{ "name": string (non-empty), "value": integer ≥ 0 }`
///     - The sum of all subsection values must equal the task `value`
/// - `total_value` (required, integer ≥ 0): Must equal the sum of all task values
///
/// ### Success Response (200 OK)
/// ```json
/// { "success": true, "message": "Mark allocator successfully saved.", "data": "{}" }
/// ```
///
/// ### Error Responses
/// - **400 Bad Request** – Invalid structure or values  
/// - **404 Not Found** – Module or assignment directory does not exist  
/// - **500 Internal Server Error** – Save failure
pub async fn save(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(req): Json<Value>,
) -> impl IntoResponse {
    let bad =
        |msg: &str| (StatusCode::BAD_REQUEST, Json(ApiResponse::<()>::error(msg))).into_response();

    // Root must be an object with "tasks" array and "total_value" integer
    let root = match req.as_object() {
        Some(o) => o,
        None => return bad("Body must be a JSON object"),
    };

    let tasks = match root.get("tasks").and_then(|t| t.as_array()) {
        Some(a) if !a.is_empty() => a,
        _ => return bad("\"tasks\" must be a non-empty array"),
    };

    let total_value = match root.get("total_value").and_then(|v| v.as_i64()) {
        Some(v) if v >= 0 => v,
        _ => return bad("\"total_value\" must be an integer >= 0"),
    };

    let mut sum_task_values: i64 = 0;

    for (idx, entry) in tasks.iter().enumerate() {
        // Each entry must be an object with exactly one key: "taskN"
        let entry_obj = match entry.as_object() {
            Some(m) if m.len() == 1 => m,
            Some(_) => {
                return bad(&format!(
                    "tasks[{}] must be an object with exactly one key (e.g., \"task1\")",
                    idx
                ));
            }
            None => return bad(&format!("tasks[{}] must be an object", idx)),
        };

        let (task_key, task_val) = entry_obj.iter().next().unwrap();

        // Validate key format: taskN, extract N
        if !task_key.starts_with("task") {
            return bad(&format!(
                "tasks[{}] invalid key '{}': expected key like \"task1\"",
                idx, task_key
            ));
        }
        let key_num_part = &task_key[4..];
        let key_task_num: i64 = match key_num_part.parse::<i64>() {
            Ok(n) if n > 0 => n,
            _ => {
                return bad(&format!(
                    "tasks[{}] invalid key '{}': expected positive integer after 'task'",
                    idx, task_key
                ));
            }
        };

        // Inner task object
        let task_obj = match task_val.as_object() {
            Some(o) => o,
            None => {
                return bad(&format!(
                    "tasks[{}].{} value must be an object",
                    idx, task_key
                ));
            }
        };

        // name
        let name = match task_obj.get("name").and_then(|v| v.as_str()) {
            Some(s) if !s.trim().is_empty() => s,
            _ => {
                return bad(&format!(
                    "tasks[{}].{}.name must be a non-empty string",
                    idx, task_key
                ));
            }
        };

        // task_number (must match key)
        let task_number = match task_obj.get("task_number").and_then(|v| v.as_i64()) {
            Some(n) if n > 0 => n,
            _ => {
                return bad(&format!(
                    "tasks[{}].{}.task_number must be a positive integer",
                    idx, task_key
                ));
            }
        };
        if task_number != key_task_num {
            return bad(&format!(
                "tasks[{}]: key '{}' does not match inner task_number {}",
                idx, task_key, task_number
            ));
        }

        // task value
        let task_value = match task_obj.get("value").and_then(|v| v.as_i64()) {
            Some(v) if v >= 0 => v,
            _ => {
                return bad(&format!(
                    "tasks[{}].{}.value must be an integer >= 0",
                    idx, task_key
                ));
            }
        };

        // subsections (required array; may be empty)
        let subsections = match task_obj.get("subsections").and_then(|v| v.as_array()) {
            Some(a) => a,
            None => {
                return bad(&format!(
                    "tasks[{}].{}.subsections must be an array (can be empty)",
                    idx, task_key
                ));
            }
        };

        // Validate subsections and sum values
        let mut sum_sub_values: i64 = 0;
        for (sidx, s) in subsections.iter().enumerate() {
            let s_obj = match s.as_object() {
                Some(o) => o,
                None => {
                    return bad(&format!(
                        "tasks[{}].{}.subsections[{}] must be an object",
                        idx, task_key, sidx
                    ));
                }
            };

            match s_obj.get("name").and_then(|v| v.as_str()) {
                Some(n) if !n.trim().is_empty() => {}
                _ => {
                    return bad(&format!(
                        "tasks[{}].{}.subsections[{}].name must be a non-empty string",
                        idx, task_key, sidx
                    ));
                }
            }

            let sub_value = match s_obj.get("value").and_then(|v| v.as_i64()) {
                Some(v) if v >= 0 => v,
                _ => {
                    return bad(&format!(
                        "tasks[{}].{}.subsections[{}].value must be an integer >= 0",
                        idx, task_key, sidx
                    ));
                }
            };

            sum_sub_values += sub_value;
        }

        if sum_sub_values != task_value {
            return bad(&format!(
                "tasks[{}] ('{}' / {}): sum of subsection values ({}) must equal task value ({})",
                idx, name, task_key, sum_sub_values, task_value
            ));
        }

        sum_task_values += task_value;
    }

    if sum_task_values != total_value {
        return bad(&format!(
            "sum of task values must equal total_value ({}), got {}",
            total_value, sum_task_values
        ));
    }

    match save_allocator(module_id, assignment_id, req).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                "{}",
                "Mark allocator successfully saved.",
            )),
        )
            .into_response(),
        Err(SaveError::DirectoryNotFound) => (
            StatusCode::NOT_FOUND,
            Json::<ApiResponse<()>>(ApiResponse::error(
                "Module or assignment directory does not exist",
            )),
        )
            .into_response(),
        Err(SaveError::JsonError(_)) => (
            StatusCode::BAD_REQUEST,
            Json::<ApiResponse<()>>(ApiResponse::error("Invalid JSON")),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json::<ApiResponse<()>>(ApiResponse::error("Could not save file")),
        )
            .into_response(),
    }
}
