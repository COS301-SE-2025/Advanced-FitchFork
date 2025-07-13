use std::fs::{self, File};
use serde_json::{Value, from_str, json};
use std::io::{self, ErrorKind, Write};
use chrono::prelude::*;
use std::path::PathBuf;
use std::env;

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
const SEPARATOR: &str = "&-=-&";
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
pub async fn generate_allocator(module: i64, assignment: i64) -> Result<Value, SaveError> {
    let base = env::var("ASSIGNMENT_STORAGE_ROOT")
        .map_err(|_| SaveError::DirectoryNotFound)?;

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

    // Prepare the mark_allocator directory path
    let allocator_dir_path = PathBuf::from(&base)
        .join(format!("module_{}", module))
        .join(format!("assignment_{}", assignment))
        .join("mark_allocator");

    // If it doesn't exist, try to create it (and any parents)
    if let Err(err) = fs::create_dir_all(&allocator_dir_path) {
        return Err(SaveError::IoError(err));
    }

    let mut task_index = 1;
    let mut tasks_json: Vec<Value> = vec![];

    for p in paths {
        let entry = p.map_err(SaveError::IoError)?;
        let file_name = entry
            .file_name()
            .into_string()
            .map_err(|_| SaveError::IoError(io::Error::new(ErrorKind::InvalidData, "Invalid filename")))?;

        let file_path = memo_output_path.join(&file_name);

        let content = fs::read_to_string(&file_path)?;
        let mut subsections = vec![];
        let mut current_section = String::new();
        let mut mark_counter = 0;
        let mut task_value = 0;

        for line in content.lines() {
            let split: Vec<_> = line.split(SEPARATOR).collect();

            if split.len() > 1 {
                if !current_section.is_empty() {
                    subsections.push(json!({
                        "name": current_section,
                        "value": mark_counter
                    }));
                    task_value += mark_counter;
                }
                current_section = split.last().unwrap().trim().to_string();
                mark_counter = 0;
            } else if !line.trim().is_empty() {
                mark_counter += 1;
            }
        }

        if !current_section.is_empty() {
            subsections.push(json!({
                "name": current_section,
                "value": mark_counter
            }));
            task_value += mark_counter;
        }

        let task_name = format!("task{}", task_index);
        let task_entry = json!({
            task_name: {
                "name": format!("Task {}", task_index),
                "value": task_value,
                "subsections": subsections
            }
        });

        tasks_json.push(task_entry);
        task_index += 1;
    }

    let now = Utc::now().to_rfc3339();
    let final_json = json!({
        "generated_at": now,
        "tasks": tasks_json
    });

    fs::create_dir_all(&allocator_dir_path)?;
    let output_path = allocator_dir_path.join("allocator.json");
    let mut file = File::create(&output_path)?;
    write!(file, "{}", serde_json::to_string_pretty(&final_json)?)?;

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
    let base = std::env::var("ASSIGNMENT_STORAGE_ROOT")
        .map_err(|_| SaveError::DirectoryNotFound)?;

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
    let base = std::env::var("ASSIGNMENT_STORAGE_ROOT")
        .map_err(|_| SaveError::DirectoryNotFound)?;

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

    Ok(())
}