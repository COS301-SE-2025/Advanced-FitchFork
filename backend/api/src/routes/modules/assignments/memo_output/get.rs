use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use std::{env, fs, path::PathBuf};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use axum::body::Body;
use axum::response::IntoResponseParts;

/// Retrieves the generated memo output file for a given assignment.
///
/// This endpoint looks for the first file in the `memo_output` directory located at:
/// `ASSIGNMENT_STORAGE_ROOT/module_{module_id}/assignment_{assignment_id}/memo_output`.
///
/// ### Returns:
/// - `200 OK` with the file stream as an attachment if a file is found
/// - `404 Not Found` if:
///     - The `memo_output` directory doesn't exist
///     - The directory exists but contains no files
/// - `500 Internal Server Error` if reading the directory or opening the file fails
///
/// ### Example `curl` request:
/// ```bash
/// curl -X GET http://localhost:3000/modules/1/assignments/2/memo/memo-output -OJ
/// ```
/// This will download the first file in the memo output directory for assignment 2 in module 1.
pub async fn get_memo_output_file(
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
            format!("Output directory does not exist for assignment {}", assignment_id),
        )
            .into_response();
    }

    let mut entries = match fs::read_dir(&output_dir) {
        Ok(entries) => entries.filter_map(Result::ok).collect::<Vec<_>>(),
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read output directory",
            )
                .into_response();
        }
    };

    let Some(file_entry) = entries.iter().find(|e| e.path().is_file()) else {
        return (
            StatusCode::NOT_FOUND,
            "No output file found in memo_output directory",
        )
            .into_response();
    };

    let file_path = file_entry.path();

    match File::open(&file_path).await {
        Ok(file) => {
            let stream = ReaderStream::new(file);
            let filename = file_path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("output.txt");

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/octet-stream")
                .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename))
                .body(Body::from_stream(stream))
                .unwrap()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to open output file",
        )
            .into_response(),
    }
}
