use crate::execution_config::ExecutionConfig;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Internal helper used by both production and test code.
/// Checks config validity and file existence based on passed `storage_root`.
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

/// Public version that reads from ASSIGNMENT_STORAGE_ROOT env var,
/// converts relative paths to absolute paths relative to project root.
pub fn validate_memo_files(module_id: i64, assignment_id: i64) -> Result<(), String> {
    let storage_root_str = std::env::var("ASSIGNMENT_STORAGE_ROOT")
        .map_err(|_| "ASSIGNMENT_STORAGE_ROOT env var is not set".to_string())?;

    let storage_root = resolve_storage_root(&storage_root_str);

    validate_memo_files_with_root(module_id, assignment_id, &storage_root)
}

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
        "language": "python"
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
}
