use crate::execution_config::ExecutionConfig;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Validates the presence of necessary `.zip` files and config correctness
/// for a given module and assignment in the specified storage root.
///
/// This function:
/// - Checks that the execution config JSON file is valid and readable.
/// - Ensures that the `memo`, `makefile`, and `main` subdirectories each
///   contain at least one `.zip` file.
///
/// # Arguments
///
/// * `module_id` - The module identifier.
/// * `assignment_id` - The assignment identifier.
/// * `storage_root` - The root path where assignment files are stored.
///
/// # Errors
///
/// Returns an `Err` with a descriptive message if:
/// - The config JSON file is missing or invalid.
/// - Any of the required directories does not contain a `.zip` file.
/// - Any directory or file cannot be read.
pub fn validate_memo_files_with_root(
    module_id: i64,
    assignment_id: i64,
    storage_root: &Path,
) -> Result<(), String> {
    // Validate config as before
    ExecutionConfig::get_execution_config_with_base(
        module_id,
        assignment_id,
        Some(storage_root.to_str().unwrap()),
    )
    .map_err(|e| format!("Config validation failed: {}", e))?;

    // These are the directories we require to have at least one zip file
    let required_dirs = ["memo", "makefile", "main"];

    for dir_name in &required_dirs {
        let dir_path = storage_root
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id))
            .join(dir_name);

        // Read directory entries
        let entries = std::fs::read_dir(&dir_path)
            .map_err(|_| format!("Failed to read directory {:?}", dir_path))?;

        // Check if there's at least one .zip file
        let mut has_zip = false;
        for entry_res in entries {
            let entry = entry_res.map_err(|_| format!("Failed to read entry in {:?}", dir_path))?;
            let path = entry.path();

            if path.extension().and_then(|ext| ext.to_str()) == Some("zip") {
                has_zip = true;
                break;
            }
        }

        if !has_zip {
            return Err(format!(
                "Required directory '{}' does not contain any .zip file at {:?}",
                dir_name, dir_path
            ));
        }
    }

    Ok(())
}

/// Validates memo files by reading the storage root directory from the
/// `ASSIGNMENT_STORAGE_ROOT` environment variable, resolving relative paths,
/// and calling `validate_memo_files_with_root`.
///
/// # Arguments
///
/// * `module_id` - The module identifier.
/// * `assignment_id` - The assignment identifier.
///
/// # Errors
///
/// Returns an `Err` if the environment variable is missing or validation fails.
pub fn validate_memo_files(module_id: i64, assignment_id: i64) -> Result<(), String> {
    let storage_root_str = std::env::var("ASSIGNMENT_STORAGE_ROOT")
        .map_err(|_| "ASSIGNMENT_STORAGE_ROOT env var is not set".to_string())?;

    let storage_root = resolve_storage_root(&storage_root_str);

    validate_memo_files_with_root(module_id, assignment_id, &storage_root)
}

/// Validates the presence of necessary `.zip` files and a submission file for a student attempt,
/// and validates the config.
///
/// This function:
/// - Checks that the execution config JSON file is valid and readable.
/// - Ensures that the `makefile` and `main` subdirectories each contain at least one `.zip` file.
/// - Ensures that the `assignment_submissions/user_{user_id}/attempt_{attempt_number}/` directory
///   contains at least one submission file (e.g., `*.txt`, `*.zip`, etc.).
///
/// # Arguments
///
/// * `module_id` - The module identifier.
/// * `assignment_id` - The assignment identifier.
/// * `user_id` - The user identifier (e.g., student).
/// * `attempt_number` - The submission attempt number.
/// * `storage_root` - The root path where assignment files are stored.
///
/// # Errors
///
/// Returns an `Err` if:
/// - The config JSON file is missing or invalid.
/// - Any required directory does not contain a `.zip` file.
/// - The user submission directory is missing or empty.
pub fn validate_submission_files_with_root(
    module_id: i64,
    assignment_id: i64,
    user_id: i64,
    attempt_number: i64,
    storage_root: &Path,
) -> Result<(), String> {
    // Validate config
    ExecutionConfig::get_execution_config_with_base(
        module_id,
        assignment_id,
        Some(storage_root.to_str().unwrap()),
    )
    .map_err(|e| format!("Config validation failed: {}", e))?;

    // Required .zip directories
    for dir_name in &["makefile", "main"] {
        let dir_path = storage_root
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id))
            .join(dir_name);

        let entries = fs::read_dir(&dir_path)
            .map_err(|_| format!("Failed to read directory {:?}", dir_path))?;

        let has_zip = entries
            .filter_map(Result::ok)
            .any(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("zip"));

        if !has_zip {
            return Err(format!(
                "Required directory '{}' does not contain any .zip file at {:?}",
                dir_name, dir_path
            ));
        }
    }

    // Submission file
    let submission_path = storage_root
        .join(format!("module_{}", module_id))
        .join(format!("assignment_{}", assignment_id))
        .join("assignment_submissions")
        .join(format!("user_{}", user_id))
        .join(format!("attempt_{}", attempt_number));

    let entries = fs::read_dir(&submission_path)
        .map_err(|_| format!("Failed to read submission directory {:?}", submission_path))?;

    let has_any_file = entries.filter_map(Result::ok).any(|e| e.path().is_file());

    if !has_any_file {
        return Err(format!("No submission file found at {:?}", submission_path));
    }

    Ok(())
}

/// Validates submission files using the `ASSIGNMENT_STORAGE_ROOT` environment variable.
pub fn validate_submission_files(
    module_id: i64,
    assignment_id: i64,
    user_id: i64,
    attempt_number: i64,
) -> Result<(), String> {
    let storage_root_str = std::env::var("ASSIGNMENT_STORAGE_ROOT")
        .map_err(|_| "ASSIGNMENT_STORAGE_ROOT env var is not set".to_string())?;

    let storage_root = resolve_storage_root(&storage_root_str);

    validate_submission_files_with_root(
        module_id,
        assignment_id,
        user_id,
        attempt_number,
        &storage_root,
    )
}

/// Resolves a potentially relative storage root path to an absolute path,
/// assuming the current working directory is the project backend/code_runner folder.
///
/// Relative paths are adjusted to point to the backend root folder.
///
/// # Arguments
///
/// * `storage_root` - The storage root path as a string.
///
/// # Returns
///
/// An absolute `PathBuf` for the storage root.
fn resolve_storage_root(storage_root: &str) -> PathBuf {
    let path = PathBuf::from(storage_root);
    if path.is_relative() {
        // Convert relative path to absolute path relative to current working directory's parent folder
        // (Assuming current dir is backend/code_runner and actual data is in backend/data/assignment_files)
        let mut abs_path = std::env::current_dir().unwrap();
        abs_path.pop(); // go up one level from backend/code_runner to backend/
        abs_path.push(path);
        abs_path
    } else {
        path
    }
}

/// Helper function to write a default execution config JSON file
/// at the expected location for a given module and assignment.
///
/// Used in tests or initialization.
///
/// # Arguments
///
/// * `base_path` - Base storage path to write config under.
/// * `module_id` - The module identifier.
/// * `assignment_id` - The assignment identifier.
pub fn write_config_json(base_path: &PathBuf, module_id: i64, assignment_id: i64) {
    let config_dir = base_path
        .join(format!("module_{}", module_id))
        .join(format!("assignment_{}", assignment_id))
        .join("config");
    fs::create_dir_all(&config_dir).unwrap();

    let config_json = r#"
    {
        "timeout_secs": 10,
        "max_memory": "256m",
        "max_cpus": "1.0",
        "max_uncompressed_size": 10485760,
        "max_processes": 16,
        "marking_scheme": "exact",
        "feedback_scheme": "auto"
    }
    "#;

    fs::write(config_dir.join("config.json"), config_json).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_required_file(
        base_path: &PathBuf,
        module_id: i64,
        assignment_id: i64,
        file_type: &str,
        filename: &str,
    ) {
        let dir = base_path
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id))
            .join(file_type);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(filename), b"dummy").unwrap();
    }

    #[test]
    fn test_validate_all_files_exist() {
        let temp = tempdir().unwrap();
        let storage_root = temp.path().to_path_buf();
        let module_id = 5;
        let assignment_id = 99;

        write_config_json(&storage_root, module_id, assignment_id);

        write_required_file(&storage_root, module_id, assignment_id, "memo", "memo.zip");
        write_required_file(
            &storage_root,
            module_id,
            assignment_id,
            "makefile",
            "makefile.zip",
        );
        write_required_file(&storage_root, module_id, assignment_id, "main", "main.zip");

        let result = validate_memo_files_with_root(module_id, assignment_id, &storage_root);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_file_causes_error() {
        let temp = tempdir().unwrap();
        let storage_root = temp.path().to_path_buf();
        let module_id = 1;
        let assignment_id = 101;

        write_config_json(&storage_root, module_id, assignment_id);

        write_required_file(&storage_root, module_id, assignment_id, "memo", "memo.zip");
        write_required_file(&storage_root, module_id, assignment_id, "main", "main.zip");

        fs::create_dir_all(
            storage_root
                .join(format!("module_{}", module_id))
                .join(format!("assignment_{}", assignment_id))
                .join("makefile"),
        )
        .unwrap();

        let result = validate_memo_files_with_root(module_id, assignment_id, &storage_root);
        println!("{:?}", result);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("makefile"));
    }

    #[test]
    fn test_invalid_config_causes_error() {
        let temp = tempdir().unwrap();
        let storage_root = temp.path().to_path_buf();
        let module_id = 2;
        let assignment_id = 123;

        let config_dir = storage_root
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id))
            .join("config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("config.json"), b"{ invalid json }").unwrap();

        write_required_file(&storage_root, module_id, assignment_id, "memo", "memo.zip");
        write_required_file(
            &storage_root,
            module_id,
            assignment_id,
            "makefile",
            "makefile.zip",
        );
        write_required_file(&storage_root, module_id, assignment_id, "main", "main.zip");

        let result = validate_memo_files_with_root(module_id, assignment_id, &storage_root);
        println!("{:?}", result);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Config validation failed"));
    }

    #[test]
    fn test_validate_submission_success() {
        let temp = tempdir().unwrap();
        let storage_root = temp.path().to_path_buf();
        let module_id = 7;
        let assignment_id = 888;
        let user_id = 42;
        let attempt_number = 2;

        write_config_json(&storage_root, module_id, assignment_id);

        write_required_file(
            &storage_root,
            module_id,
            assignment_id,
            "makefile",
            "mf.zip",
        );
        write_required_file(&storage_root, module_id, assignment_id, "main", "main.zip");

        let submission_dir = storage_root
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id))
            .join("assignment_submissions")
            .join(format!("user_{}", user_id))
            .join(format!("attempt_{}", attempt_number));
        fs::create_dir_all(&submission_dir).unwrap();
        fs::write(submission_dir.join("482.txt"), b"submission").unwrap();

        let result = validate_submission_files_with_root(
            module_id,
            assignment_id,
            user_id,
            attempt_number,
            &storage_root,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_submission_missing_file() {
        let temp = tempdir().unwrap();
        let storage_root = temp.path().to_path_buf();
        let module_id = 7;
        let assignment_id = 999;
        let user_id = 1;
        let attempt_number = 1;

        write_config_json(&storage_root, module_id, assignment_id);

        write_required_file(
            &storage_root,
            module_id,
            assignment_id,
            "makefile",
            "mf.zip",
        );
        write_required_file(&storage_root, module_id, assignment_id, "main", "main.zip");

        // Missing submission dir

        let result = validate_submission_files_with_root(
            module_id,
            assignment_id,
            user_id,
            attempt_number,
            &storage_root,
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Failed to read submission directory")
        );
    }
}
