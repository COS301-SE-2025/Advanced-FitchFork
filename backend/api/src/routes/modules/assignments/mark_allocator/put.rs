use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use crate::response::ApiResponse;
use serde_json::Value;
use util::mark_allocator::mark_allocator::{save_allocator, SaveError};

const EPS: f64 = 1e-6;

/// PUT /api/modules/{module_id}/assignments/{assignment_id}/mark_allocator
///
/// Save the mark allocator JSON configuration for a specific assignment. Accessible to users with
/// Lecturer roles assigned to the module.
///
/// This endpoint saves a mark allocator configuration to the assignment directory. The configuration
/// defines how many **points** are allocated to each task and its subsections. The saved configuration
/// is used by the grading system to calculate final marks.
///
/// ### Path Parameters
/// - `module_id` (i64): The ID of the module containing the assignment
/// - `assignment_id` (i64): The ID of the assignment to save the mark allocator for
///
/// ### Request Body (points-based schema)
/// A JSON object with a `tasks` array. Each element of `tasks` is an object with exactly one key of
/// the form `"taskN"` (N is a positive integer), whose value specifies the task:
///
/// ```json
/// {
///   "generated_at": "2025-08-11T00:56:24.487Z",
///   "tasks": [
///     {
///       "task1": {
///         "name": "Task 1",
///         "subsections": [
///           { "name": "Correctness", "value": 5 },
///           { "name": "Quality",     "value": 3 },
///           { "name": "Docs",        "value": 2 }
///         ],
///         "value": 10
///       }
///     },
///     {
///       "task2": {
///         "name": "Task 2",
///         "subsections": [
///           { "name": "Part A", "value": 6 },
///           { "name": "Part B", "value": 4 }
///         ],
///         "value": 10
///       }
///     }
///   ]
/// }
/// ```
///
/// #### Field semantics
/// - `generated_at` (optional, string): ISO-8601 timestamp indicating when the allocator was generated.
/// - `tasks` (required, non-empty array): Each element must be an object with exactly one `"taskN"` key.
///   - `"taskN"` (required): N is a positive integer (1, 2, 3, ...).
///     - `name` (required, non-empty string): Task display name.
///     - `value` (required, number ≥ 0): Total points for the task.
///     - `subsections` (required, non-empty array):
///       - Each subsection is an object: `{ "name": string (non-empty), "value": number ≥ 0 }`.
///       - The sum of all subsection `value`s must equal the task `value` (within 1e-6).
///
/// ### Example Request
/// ```bash
/// curl -X PUT http://localhost:3000/api/modules/1/assignments/2/mark_allocator \
///   -H "Authorization: Bearer <token>" \
///   -H "Content-Type": "application/json" \
///   -d '{
///     "generated_at": "2025-08-11T01:10:00.000Z",
///     "tasks": [
///       {
///         "task2": {
///           "name": "Task 2",
///           "subsections": [
///             { "name": "Correctness", "value": 5 },
///             { "name": "Quality",     "value": 3 },
///             { "name": "Docs",        "value": 2 }
///           ],
///           "value": 10
///         }
///       }
///     ]
///   }'
/// ```
///
/// ### Success Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "message": "Mark allocator successfully saved.",
///   "data": "{}"
/// }
/// ```
///
/// ### Error Responses
/// **400 Bad Request** – Invalid structure or values
/// ```json
/// { "success": false, "message": "Invalid mark allocator structure or values" }
/// ```
///
/// **404 Not Found** – Module or assignment directory does not exist
/// ```json
/// { "success": false, "message": "Module or assignment directory does not exist" }
/// ```
///
/// **500 Internal Server Error** – Save failure
/// ```json
/// { "success": false, "message": "Could not save file" }
/// ```
///
/// ### Validation Rules
/// - Request body must be a JSON object with a non-empty `tasks` array.
/// - Each element of `tasks` must be an object with exactly one key named `"taskN"` where N is a positive integer.
/// - Each task must include `name` (non-empty string), `value` (number ≥ 0), and a non-empty `subsections` array.
/// - Each subsection must include `name` (non-empty string) and `value` (number ≥ 0).
/// - For each task, `sum(subsections[].value) == task.value` within a tolerance of `1e-6`.
/// - User must have Lecturer permissions for the module.
///
/// ### Notes
/// - The mark allocator configuration is saved as a JSON file in the assignment directory.
/// - This endpoint overwrites any existing mark allocator for the assignment.
/// - This API uses a **points-based** schema (no weights/percentages).
pub async fn save(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
    Json(req): Json<Value>,
) -> impl IntoResponse {
    let bad = |msg: &str| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(msg)),
        )
            .into_response()
    };

    // Root must be an object with "tasks" array
    let root = match req.as_object() {
        Some(o) => o,
        None => return bad("Body must be a JSON object"),
    };

    let tasks = match root.get("tasks").and_then(|t| t.as_array()) {
        Some(a) if !a.is_empty() => a,
        _ => return bad("\"tasks\" must be a non-empty array"),
    };

    // Validate each { "taskN": { name, value, subsections[] } }
    for (idx, item) in tasks.iter().enumerate() {
        let obj = match item.as_object() {
            Some(m) if m.len() == 1 => m,
            _ => return bad(&format!("tasks[{idx}] must be an object with exactly one key")),
        };

        // Extract ("taskN", body)
        let (task_key, body) = obj.iter().next().unwrap();

        // task key must be taskN where N is a positive integer
        if !task_key.starts_with("task") {
            return bad(&format!("tasks[{idx}] key must start with \"task\""));
        }
        let num_part = &task_key[4..];
        if num_part.is_empty() || num_part.parse::<u32>().is_err() {
            return bad(&format!("tasks[{idx}] key must be of the form \"taskN\" with numeric N"));
        }

        // body must be object with name, value, subsections
        let body_obj = match body.as_object() {
            Some(b) => b,
            None => return bad(&format!("tasks[{idx}].{task_key} must be an object")),
        };

        let name = match body_obj.get("name").and_then(|v| v.as_str()) {
            Some(s) if !s.trim().is_empty() => s,
            _ => return bad(&format!("tasks[{idx}].{task_key}.name must be a non-empty string")),
        };

        let value = match body_obj.get("value").and_then(|v| v.as_f64()) {
            Some(v) if v >= 0.0 => v,
            _ => return bad(&format!("tasks[{idx}].{task_key}.value must be a number >= 0")),
        };

        let subsections = match body_obj.get("subsections").and_then(|v| v.as_array()) {
            Some(a) if !a.is_empty() => a,
            _ => return bad(&format!("tasks[{idx}].{task_key}.subsections must be a non-empty array")),
        };

        // Validate each subsection and sum values
        let mut sum_vals = 0.0;
        for (sidx, s) in subsections.iter().enumerate() {
            let s_obj = match s.as_object() {
                Some(o) => o,
                None => {
                    return bad(&format!(
                        "tasks[{idx}].{task_key}.subsections[{sidx}] must be an object"
                    ))
                }
            };

            match s_obj.get("name").and_then(|v| v.as_str()) {
                Some(sn) if !sn.trim().is_empty() => (),
                _ => {
                    return bad(&format!(
                        "tasks[{idx}].{task_key}.subsections[{sidx}].name must be a non-empty string"
                    ))
                }
            }

            let sval = match s_obj.get("value").and_then(|v| v.as_f64()) {
                Some(v) if v >= 0.0 => v,
                _ => {
                    return bad(&format!(
                        "tasks[{idx}].{task_key}.subsections[{sidx}].value must be a number >= 0"
                    ))
                }
            };
            sum_vals += sval;
        }

        if (sum_vals - value).abs() > EPS {
            return bad(&format!(
                "tasks[{idx}].{task_key}.subsections values must sum to {} (got {})",
                value, sum_vals
            ));
        }

        // Optional: additional invariants can go here
        let _ = name; // silence unused variable if not used further
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
