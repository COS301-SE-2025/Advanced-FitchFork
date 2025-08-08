//utils/config_management.rs
use std::fs;
use std::path::Path;
use util::execution_config::ExecutionConfig;

/// Loads an `ExecutionConfig` from a given JSON file path.
///
/// # Arguments
/// - `path`: A path to a `.json` file.
///
/// # Returns
/// - `Ok(ExecutionConfig)` if the file is found and successfully parsed.
/// - `Err(String)` with a descriptive error message otherwise.
pub fn load_execution_config_from_json<P: AsRef<Path>>(path: P) -> Result<ExecutionConfig, String> {
    let path_ref = path.as_ref();

    // Check that it's a file and ends with .json
    if !path_ref.exists() || !path_ref.is_file() {
        return Err(format!(
            "Config file {:?} does not exist or is not a valid file",
            path_ref
        ));
    }

    let content = fs::read_to_string(path_ref)
        .map_err(|e| format!("Failed to read config file {:?}: {}", path_ref, e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON in config file {:?}: {}", path_ref, e))
}
