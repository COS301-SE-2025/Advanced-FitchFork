use crate::execution_config::ExecutionConfig;
use crate::paths::{mark_allocator_dir, mark_allocator_path};
use serde_json::{Value, from_str, json};
use std::fs::{self, File};
use std::io::{self, ErrorKind, Write};
use std::path::PathBuf;

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

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: i64,
    pub task_number: i64,
    pub code_coverage: bool,
    pub name: String,
}

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
pub async fn generate_allocator(
    module: i64,
    assignment: i64,
    tasks_info: &[(TaskInfo, PathBuf)],
) -> Result<Value, SaveError> {
    let separator = ExecutionConfig::get_execution_config(module, assignment)
        .map(|config| config.marking.deliminator)
        .unwrap_or_else(|_| "&-=-&".to_string());

    let allocator_dir_path = mark_allocator_dir(module, assignment);
    fs::create_dir_all(&allocator_dir_path).map_err(SaveError::IoError)?;

    let mut tasks_json = vec![];
    let mut total_value = 0;

    for (info, maybe_path) in tasks_info {
        let mut subsections = vec![];
        let mut task_value = 0;

        if info.code_coverage {
            // skip coverage tasks from point allocation
        } else if maybe_path.exists() {
            let content = fs::read_to_string(maybe_path)?;

            let mut current_section = String::new();
            let mut mark_counter = 0;

            for line in content.lines() {
                let split: Vec<_> = line.split(&separator).collect();
                if split.len() > 1 {
                    if !current_section.is_empty() {
                        subsections.push(json!({ "name": current_section, "value": mark_counter }));
                        task_value += mark_counter;
                    }
                    current_section = split.last().unwrap().trim().to_string();
                    mark_counter = 0;
                } else if !line.trim().is_empty() {
                    mark_counter += 1;
                }
            }

            if !current_section.is_empty() {
                subsections.push(json!({ "name": current_section, "value": mark_counter }));
                task_value += mark_counter;
            }
        }

        let task_key = format!("task{}", info.task_number);
        let task_body = json!({
            "name": info.name.clone(),
            "task_number": info.task_number,
            "value": task_value,
            "subsections": subsections,
            "code_coverage": info.code_coverage
        });
        tasks_json.push(json!({ task_key: task_body }));
        total_value += task_value;
    }

    let now = chrono::Utc::now().to_rfc3339();
    let final_json = json!({
        "generated_at": now,
        "tasks": tasks_json,
        "total_value": total_value
    });

    let output_path = mark_allocator_path(module, assignment);
    let mut file = File::create(&output_path)?;
    write!(file, "{}", serde_json::to_string_pretty(&final_json)?)?;
    file.flush()?;

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
    let path = mark_allocator_path(module, assignment);

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
    let dir_path = mark_allocator_dir(module, assignment);
    fs::create_dir_all(&dir_path)?;

    let file_path = mark_allocator_path(module, assignment);

    let mut file = File::create(&file_path)?;
    let content = serde_json::to_string_pretty(&json)?;
    file.write_all(content.as_bytes())?;
    file.flush()?;

    Ok(())
}
