use crate::execution_config::ExecutionConfig;
use std::path::PathBuf;

/// Internal helper used by both production and test code.
/// Checks config validity and file existence based on passed `storage_root`.
pub fn validate_memo_files_with_root(
    module_id: i64,
    assignment_id: i64,
    storage_root: &str,
) -> Result<(), String> {
    ExecutionConfig::get_execution_config_with_base(assignment_id, Some(storage_root))
        .map_err(|e| format!("Config validation failed: {}", e))?;

    let required_files = [
        ("memo", "memo.zip"),
        ("makefile", "makefile.zip"),
        ("main", "main.zip"),
    ];

    for (file_type, filename) in required_files.iter() {
        let path = PathBuf::from(storage_root)
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id))
            .join(file_type)
            .join(filename);

        if !path.exists() {
            return Err(format!("Required file missing: {}", path.display()));
        }
    }

    Ok(())
}

/// Public version that reads from ASSIGNMENT_STORAGE_ROOT
pub fn validate_memo_files(module_id: i64, assignment_id: i64) -> Result<(), String> {
    let storage_root = std::env::var("ASSIGNMENT_STORAGE_ROOT")
        .map_err(|_| "ASSIGNMENT_STORAGE_ROOT env var is not set".to_string())?;

    validate_memo_files_with_root(module_id, assignment_id, &storage_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn write_config_json(base_path: &PathBuf, assignment_id: i64) {
        let config_dir = base_path
            .join("assignments")
            .join(assignment_id.to_string());
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

        // Setup valid config
        let config_dir = storage_root
            .join("assignments")
            .join(assignment_id.to_string());
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("config.json"),
            r#"
            {
                "timeout_secs": 10,
                "max_memory": "256m",
                "max_cpus": "1.0",
                "max_uncompressed_size": 10485760,
                "max_processes": 16,
                "language": "python"
            }
            "#,
        )
        .unwrap();

        // Write required zip files
        write_required_file(&storage_root, module_id, assignment_id, "memo", "memo.zip");
        write_required_file(
            &storage_root,
            module_id,
            assignment_id,
            "makefile",
            "makefile.zip",
        );
        write_required_file(&storage_root, module_id, assignment_id, "main", "main.zip");

        let result =
            validate_memo_files_with_root(module_id, assignment_id, storage_root.to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_file_causes_error() {
        let temp = tempdir().unwrap();
        let storage_root = temp.path().to_path_buf();
        let module_id = 1;
        let assignment_id = 101;

        // Write valid config
        write_config_json(&storage_root, assignment_id);

        // Only write 2 of the 3 required zip files
        write_required_file(&storage_root, module_id, assignment_id, "memo", "memo.zip");
        write_required_file(&storage_root, module_id, assignment_id, "main", "main.zip");
        // makefile.zip is missing

        let result =
            validate_memo_files_with_root(module_id, assignment_id, storage_root.to_str().unwrap());
        println!("{:?}", result);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("makefile.zip"));
    }

    #[test]
    fn test_invalid_config_causes_error() {
        let temp = tempdir().unwrap();
        let storage_root = temp.path().to_path_buf();
        let module_id = 2;
        let assignment_id = 123;

        // Invalid config
        let config_dir = storage_root
            .join("assignments")
            .join(assignment_id.to_string());
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("config.json"), b"{ invalid json }").unwrap();

        // Required files
        write_required_file(&storage_root, module_id, assignment_id, "memo", "memo.zip");
        write_required_file(
            &storage_root,
            module_id,
            assignment_id,
            "makefile",
            "makefile.zip",
        );
        write_required_file(&storage_root, module_id, assignment_id, "main", "main.zip");

        let result =
            validate_memo_files_with_root(module_id, assignment_id, storage_root.to_str().unwrap());
        println!("{:?}", result);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Config validation failed"));
    }
}
