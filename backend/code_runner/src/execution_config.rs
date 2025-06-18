use serde::Deserialize;
use std::{fs, path::PathBuf};

/// Configuration for runtime environment limits (used for Docker container).
#[derive(Debug, Clone, Deserialize)]
pub struct ExecutionConfig {
    pub timeout_secs: u64,          // Max execution time
    pub max_memory: String,         // Max memory (e.g., "128m")
    pub max_cpus: String,           // CPU cores (e.g., "1.0")
    pub max_uncompressed_size: u64, // Max total size of decompressed files (zip bomb protection)
    pub max_processes: u32,         // Max number of processes inside container
    pub language: String,           // Programming language identifier
}

impl ExecutionConfig {
    /// Attempts to load a config from a `config.json` file for the given assignment.
    /// Returns `None` if the file does not exist or cannot be parsed.
    pub fn get_execution_config(assignment_id: i64) -> Option<Self> {
        let base_path = std::env::var("STORAGE_ROOT").ok()?; // e.g., "data/"
        let file_path = PathBuf::from(base_path)
            .join("assignments")
            .join(assignment_id.to_string())
            .join("config.json");

        let file_contents = fs::read_to_string(file_path).ok()?;
        serde_json::from_str(&file_contents).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_load_valid_config() {
        let temp_dir = tempdir().unwrap();
        let storage_root = temp_dir.path().to_path_buf();

        // Simulate environment setup
        env::set_var("STORAGE_ROOT", &storage_root);

        // Create path: /assignments/42/config.json
        let assignment_id = 42;
        let assignment_path = storage_root
            .join("assignments")
            .join(assignment_id.to_string());
        fs::create_dir_all(&assignment_path).unwrap();

        let config_json = r#"
        {
            "timeout_secs": 15,
            "max_memory": "256m",
            "max_cpus": "1.5",
            "max_uncompressed_size": 10485760,
            "max_processes": 64,
            "language": "python"
        }
        "#;

        let config_path = assignment_path.join("config.json");
        fs::write(config_path, config_json).unwrap();

        let config = ExecutionConfig::from_assignment_id(assignment_id);
        assert!(config.is_some());
        let cfg = config.unwrap();
        assert_eq!(cfg.timeout_secs, 15);
        assert_eq!(cfg.max_memory, "256m");
        assert_eq!(cfg.max_cpus, "1.5");
        assert_eq!(cfg.max_uncompressed_size, 10 * 1024 * 1024);
        assert_eq!(cfg.max_processes, 64);
        assert_eq!(cfg.language, "python");
    }

    #[test]
    fn test_config_file_missing() {
        let temp_dir = tempdir().unwrap();
        env::set_var("STORAGE_ROOT", temp_dir.path());

        let assignment_id = 999; // No file for this one
        let config = ExecutionConfig::from_assignment_id(assignment_id);
        assert!(config.is_none());
    }

    #[test]
    fn test_invalid_config_json() {
        let temp_dir = tempdir().unwrap();
        env::set_var("STORAGE_ROOT", temp_dir.path());

        let assignment_id = 77;
        let assignment_path = temp_dir
            .path()
            .join("assignments")
            .join(assignment_id.to_string());
        fs::create_dir_all(&assignment_path).unwrap();

        let invalid_json = r#"{ "timeout_secs": "oops" }"#;
        fs::write(assignment_path.join("config.json"), invalid_json).unwrap();

        let config = ExecutionConfig::from_assignment_id(assignment_id);
        assert!(config.is_none());
    }
}
