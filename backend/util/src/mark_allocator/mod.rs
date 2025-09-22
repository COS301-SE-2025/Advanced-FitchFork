use crate::execution_config::ExecutionConfig;
use crate::paths::{mark_allocator_dir, mark_allocator_path};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty, to_value, Value};
use std::collections::BTreeMap;
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

/// Typed representation of a subsection/value pair within a task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Subsection {
    pub name: String,
    pub value: i64,
    #[serde(default)]
    pub feedback: String,
    #[serde(default)]
    pub regex: Vec<String>,
}

/// Typed representation of a single task's allocation details.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskBody {
    pub name: String,
    pub task_number: i64,
    pub value: i64,
    pub subsections: Vec<Subsection>,
    pub code_coverage: bool,
}

/// To remain backward compatible with the existing JSON which encodes each task
/// as an object keyed by a dynamic string like "task1", we model `tasks` as a
/// vector of one-entry maps. Each map's single key is the dynamic task key and
/// the value is the `TaskBody`.
pub type TaskEntry = BTreeMap<String, TaskBody>;

/// Full allocator structure as saved to allocator.json
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Allocator {
    pub generated_at: String,
    pub tasks: Vec<TaskEntry>,
    pub total_value: i64,
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
    // Build using typed structs, then convert to Value for backward compatibility
    let typed = generate_allocator_typed(module, assignment, tasks_info).await?;

    // Persist to disk (already done inside typed variant), then return as Value
    Ok(to_value(typed)?)
}

/// Typed variant of `generate_allocator` that returns an `Allocator` and writes it to disk.
pub async fn generate_allocator_typed(
    module: i64,
    assignment: i64,
    tasks_info: &[(TaskInfo, PathBuf)],
) -> Result<Allocator, SaveError> {
    let separator = ExecutionConfig::get_execution_config(module, assignment)
        .map(|config| config.marking.deliminator)
        .unwrap_or_else(|_| "&-=-&".to_string());

    let allocator_dir_path = mark_allocator_dir(module, assignment);
    fs::create_dir_all(&allocator_dir_path).map_err(SaveError::IoError)?;

    let mut tasks_vec: Vec<TaskEntry> = vec![];
    let mut total_value: i64 = 0;

    for (info, maybe_path) in tasks_info {
        let mut subsections: Vec<Subsection> = vec![];
        let mut task_value: i64 = 0;

        if info.code_coverage {
            // skip coverage tasks from point allocation
        } else if maybe_path.exists() {
            let content = fs::read_to_string(maybe_path)?;

            let mut current_section = String::new();
            let mut mark_counter: i64 = 0;

            for line in content.lines() {
                let split: Vec<_> = line.split(&separator).collect();
                if split.len() > 1 {
                    if !current_section.is_empty() {
                        subsections.push(Subsection {
                            name: current_section,
                            value: mark_counter,
                            feedback: String::new(),
                            regex: Vec::new(),
                        });
                        task_value += mark_counter;
                    }
                    current_section = split.last().unwrap().trim().to_string();
                    mark_counter = 0;
                } else if !line.trim().is_empty() {
                    mark_counter += 1;
                }
            }

            if !current_section.is_empty() {
                subsections.push(Subsection {
                    name: current_section,
                    value: mark_counter,
                    feedback: String::new(),
                    regex: Vec::new(),
                });
                task_value += mark_counter;
            }
        }

        let task_key = format!("task{}", info.task_number);
        let body = TaskBody {
            name: info.name.clone(),
            task_number: info.task_number,
            value: task_value,
            subsections,
            code_coverage: info.code_coverage,
        };

        let mut map: TaskEntry = BTreeMap::new();
        map.insert(task_key, body);
        tasks_vec.push(map);
        total_value += task_value;
    }

    let now = chrono::Utc::now().to_rfc3339();
    let allocator = Allocator {
        generated_at: now,
        tasks: tasks_vec,
        total_value,
    };

    let output_path = mark_allocator_path(module, assignment);
    let mut file = File::create(&output_path)?;
    write!(file, "{}", to_string_pretty(&allocator)?)?;
    file.flush()?;

    Ok(allocator)
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
    let typed = load_allocator_typed(module, assignment).await?;
    Ok(to_value(typed)?)
}

/// Typed variant of `load_allocator` returning an `Allocator`.
pub async fn load_allocator_typed(module: i64, assignment: i64) -> Result<Allocator, SaveError> {
    let path = mark_allocator_path(module, assignment);

    let json_str = fs::read_to_string(&path).map_err(|err| {
        if err.kind() == ErrorKind::NotFound {
            SaveError::DirectoryNotFound
        } else {
            SaveError::IoError(err)
        }
    })?;

    let allocator = from_str::<Allocator>(&json_str)?;
    Ok(allocator)
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

/// Typed variant of `save_allocator` taking an `Allocator`.
pub async fn save_allocator_typed(
    module: i64,
    assignment: i64,
    allocator: &Allocator,
) -> Result<(), SaveError> {
    let dir_path = mark_allocator_dir(module, assignment);
    fs::create_dir_all(&dir_path)?;

    let file_path = mark_allocator_path(module, assignment);

    let mut file = File::create(&file_path)?;
    let content = to_string_pretty(allocator)?;
    file.write_all(content.as_bytes())?;
    file.flush()?;

    Ok(())
}
