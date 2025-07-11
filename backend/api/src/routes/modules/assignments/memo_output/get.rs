use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::{env, fs, path::PathBuf};
use tokio::fs as tokio_fs;
use crate::response::ApiResponse;
use serde::Serialize;

#[derive(Serialize)]
struct MemoSubsection {
    label: String,
    output: String,
}

#[derive(Serialize)]
struct MemoTaskOutput {
    task_number: i32,
    name: String,
    subsections: Vec<MemoSubsection>,
    raw: String, // NEW: full unformatted text from the file
}

/// Retrieves all memo output files for a given assignment and parses them into structured format.
///
/// This endpoint scans the `memo_output` directory for the assignment and parses all files into
/// structured tasks, where each task contains labeled subsections and a raw file representation.
///
/// Directory structure:  
/// `ASSIGNMENT_STORAGE_ROOT/module_{module_id}/assignment_{assignment_id}/memo_output`
///
/// ### Returns:
/// - `200 OK` with a JSON array of `MemoTaskOutput`
/// - `404 Not Found` if:
///     - The memo output directory doesn't exist
///     - No valid files are found in the directory
/// - `500 Internal Server Error` if reading the directory fails
///
/// ### Example `curl` request:
/// ```bash
/// curl http://localhost:3000/api/modules/1/assignments/2/memo-output
/// ```
pub async fn get_all_memo_outputs(
    Path((module_id, assignment_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    let base_path = env::var("ASSIGNMENT_STORAGE_ROOT")
        .unwrap_or_else(|_| "data/assignment_files".into());

    let output_dir = PathBuf::from(base_path)
        .join(format!("module_{}", module_id))
        .join(format!("assignment_{}", assignment_id))
        .join("memo_output");

    if !output_dir.is_dir() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<Vec<MemoTaskOutput>>::error("Memo output directory does not exist")),
        );
    }

    let entries = match fs::read_dir(&output_dir) {
        Ok(entries) => entries.filter_map(Result::ok).collect::<Vec<_>>(),
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<Vec<MemoTaskOutput>>::error("Failed to read memo output directory")),
            );
        }
    };

    let mut tasks: Vec<MemoTaskOutput> = Vec::new();

    for (i, entry) in entries.iter().enumerate() {
        let path = entry.path();
        if path.is_file() {
            let raw_content = match tokio_fs::read_to_string(&path).await {
                Ok(c) => c,
                Err(_) => continue,
            };

            let sections = raw_content.split("&-=-&").filter(|s| !s.trim().is_empty());

            let subsections = sections
                .map(|s| {
                    let mut lines = s.lines();
                    let label = lines.next().unwrap_or("").trim().to_string();
                    let output = lines.collect::<Vec<_>>().join("\n");
                    MemoSubsection { label, output }
                })
                .collect::<Vec<_>>();

            if !subsections.is_empty() {
                tasks.push(MemoTaskOutput {
                    task_number: (i + 1) as i32,
                    name: format!("Task {}", i + 1),
                    subsections,
                    raw: raw_content,
                });
            }
        }
    }

    if tasks.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<Vec<MemoTaskOutput>>::error("No memo output found")),
        );
    }

    (
        StatusCode::OK,
        Json(ApiResponse::success(tasks, "Fetched memo output successfully")),
    )
}
