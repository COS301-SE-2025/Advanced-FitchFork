use crate::execution_config::ExecutionConfig;
use chrono::prelude::*;
use serde_json::{Value, from_str, json};
use std::env;
use std::fs::{self, File};
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf}; // or adjust path if needed

pub enum SaveError {
    DirectoryNotFound,
    IoError(io::Error),
    JsonError(serde_json::Error),
}

impl From<io::Error> for SaveError {
    fn from(err: io::Error) -> Self {
        SaveError::IoError(err)
    }
}

impl From<serde_json::Error> for SaveError {
    fn from(err: serde_json::Error) -> Self {
        SaveError::JsonError(err)
    }
}

#[allow(dead_code)]
/// Generates a mark allocator JSON structure by reading memo output files from
/// the specified module and assignment directories.
///
/// Reads all files in the `memo_output` folder corresponding to the module and assignment,
/// parses lines to count marks and subsections, and constructs a JSON structure with
/// tasks and their subsections. The resulting JSON is saved to an `allocator.json` file
/// in the `mark_allocator` folder.
///
/// # Arguments
///
/// * `module` - The module id.
/// * `assignment` - The assignment id.
///
/// # Returns
///
/// * `Ok(Value)` - A JSON `Value` representing the generated allocator data.
/// * `Err(SaveError)` - An error if directory/file operations or JSON serialization fail.
///
/// # JSON Schema
///
/// ```json
/// {
///   "generated_at": "2025-08-17T22:00:00Z",
///   "tasks": [
///     {
///       "task_number": 1,
///       "value": 10,
///       "subsections": [
///         { "name": "Correctness", "value": 6 },
///         { "name": "Style", "value": 4 }
///       ]
///     }
///   ],
///   "total_value": 10
/// }
/// ```
#[allow(dead_code)]
pub async fn generate_allocator(module: i64, assignment: i64) -> Result<Value, SaveError> {
    let base = env::var("ASSIGNMENT_STORAGE_ROOT").map_err(|_| SaveError::DirectoryNotFound)?;

    // Delimiter from execution config (fallback if missing)
    let separator = ExecutionConfig::get_execution_config(module, assignment)
        .map(|config| config.marking.deliminator)
        .unwrap_or_else(|_| "&-=-&".to_string());

    let memo_output_path = PathBuf::from(&base)
        .join(format!("module_{}", module))
        .join(format!("assignment_{}", assignment))
        .join("memo_output");

    let paths = fs::read_dir(&memo_output_path).map_err(|err| {
        if err.kind() == ErrorKind::NotFound {
            SaveError::DirectoryNotFound
        } else {
            SaveError::IoError(err)
        }
    })?;

    // Ensure mark_allocator output dir exists
    let allocator_dir_path = PathBuf::from(&base)
        .join(format!("module_{}", module))
        .join(format!("assignment_{}", assignment))
        .join("mark_allocator");
    fs::create_dir_all(&allocator_dir_path).map_err(SaveError::IoError)?;

    let mut task_index: i64 = 1;
    let mut tasks_json: Vec<Value> = vec![];
    let mut total_value: i64 = 0;

    for p in paths {
        let entry = p.map_err(SaveError::IoError)?;
        let file_name = entry.file_name().into_string().map_err(|_| {
            SaveError::IoError(io::Error::new(ErrorKind::InvalidData, "Invalid filename"))
        })?;
        let file_path = memo_output_path.join(&file_name);

        // Derive task display name from file stem; fallback to "Task N"
        let task_name = Path::new(&file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("Task {}", task_index));

        // Parse memo file into subsections via delimiter; count lines between delimiters as points
        let content = fs::read_to_string(&file_path).map_err(SaveError::IoError)?;
        let mut subsections = vec![];
        let mut current_section = String::new();
        let mut mark_counter: i64 = 0;
        let mut task_value: i64 = 0;

        for line in content.lines() {
            let split: Vec<_> = line.split(&separator).collect();

            if split.len() > 1 {
                // Close previous subsection
                if !current_section.is_empty() {
                    subsections.push(json!({
                        "name": current_section,
                        "value": mark_counter
                    }));
                    task_value += mark_counter;
                }
                // Start new subsection with the trailing label after the delimiter
                current_section = split.last().unwrap().trim().to_string();
                mark_counter = 0;
            } else if !line.trim().is_empty() {
                // Count non-empty lines as points
                mark_counter += 1;
            }
        }

        // Flush final subsection
        if !current_section.is_empty() {
            subsections.push(json!({
                "name": current_section,
                "value": mark_counter
            }));
            task_value += mark_counter;
        }

        // Build the single-key task object: { "taskN": { name, task_number, value, subsections } }
        let task_key = format!("task{}", task_index);
        let task_body = json!({
            "name": task_name,
            "task_number": task_index,
            "value": task_value,
            "subsections": subsections
        });
        let task_entry = json!({ task_key: task_body });

        tasks_json.push(task_entry);
        total_value += task_value;
        task_index += 1;
    }

    let now = Utc::now().to_rfc3339();
    let final_json = json!({
        "generated_at": now,
        "tasks": tasks_json,
        "total_value": total_value
    });

    let output_path = allocator_dir_path.join("allocator.json");
    let mut file = File::create(&output_path).map_err(SaveError::IoError)?;
    write!(file, "{}", serde_json::to_string_pretty(&final_json)?).map_err(SaveError::IoError)?;
    file.flush().map_err(SaveError::IoError)?;

    Ok(final_json)
}

/// Loads the allocator JSON file for the given module and assignment.
///
/// # Arguments
///
/// * `module` - The module id.
/// * `assignment` - The assignment id.
///
/// # Returns
///
/// * `Ok(Value)` - The parsed JSON allocator data.
/// * `Err(SaveError)` - If the file or directory does not exist or parsing JSON fails.
pub async fn load_allocator(module: i64, assignment: i64) -> Result<Value, SaveError> {
    let base =
        std::env::var("ASSIGNMENT_STORAGE_ROOT").map_err(|_| SaveError::DirectoryNotFound)?;

    let path = PathBuf::from(&base)
        .join(format!("module_{}", module))
        .join(format!("assignment_{}", assignment))
        .join("mark_allocator")
        .join("allocator.json");

    let json_str = fs::read_to_string(&path).map_err(|err| {
        if err.kind() == ErrorKind::NotFound {
            SaveError::DirectoryNotFound
        } else {
            SaveError::IoError(err)
        }
    })?;

    let json_value = from_str::<Value>(&json_str)?;
    Ok(json_value)
}

/// Saves a JSON allocator object to the allocator.json file for the specified module and assignment.
/// This will overwrite any existing allocator.json file at the target path.
///
/// # Arguments
///
/// * `module` - The module number.
/// * `assignment` - The assignment number.
/// * `json` - The JSON data to save.
///
/// # Returns
///
/// * `Ok(())` - On successful write.
/// * `Err(SaveError)` - If file creation fails or JSON serialization fails.
pub async fn save_allocator(module: i64, assignment: i64, json: Value) -> Result<(), SaveError> {
    let base =
        std::env::var("ASSIGNMENT_STORAGE_ROOT").map_err(|_| SaveError::DirectoryNotFound)?;

    let dir_path = PathBuf::from(&base)
        .join(format!("module_{}", module))
        .join(format!("assignment_{}", assignment))
        .join("mark_allocator");

    // Ensure the directory exists
    fs::create_dir_all(&dir_path)?;

    let file_path = dir_path.join("allocator.json");

    let mut file = File::create(&file_path)?;
    let content = serde_json::to_string_pretty(&json)?;
    file.write_all(content.as_bytes())?;
    file.flush()?;

    Ok(())
}
