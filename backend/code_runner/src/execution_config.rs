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
    /// Load execution config from JSON file based on assignment ID.
    /// If `base_path` is None, fallback to STORAGE_ROOT env var.
    pub fn get_execution_config_with_base(
        assignment_id: i64,
        base_path: Option<&str>,
    ) -> Result<Self, String> {
        let base_path = match base_path {
            Some(path) => path.to_string(),
            None => {
                std::env::var("STORAGE_ROOT").map_err(|_| "STORAGE_ROOT not set".to_string())?
            }
        };

        let file_path = PathBuf::from(base_path)
            .join("assignments")
            .join(assignment_id.to_string())
            .join("config.json");

        let file_contents = fs::read_to_string(&file_path)
            .map_err(|_| format!("Failed to read config file at {:?}", file_path))?;

        serde_json::from_str(&file_contents).map_err(|_| "Invalid config JSON format".to_string())
    }

    pub fn get_execution_config(assignment_id: i64) -> Result<Self, String> {
        Self::get_execution_config_with_base(assignment_id, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_load_valid_config() {
        let temp_dir = tempdir().unwrap();

        let assignment_id = 42;
        let assignment_path = temp_dir
            .path()
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

        // Pass the base path explicitly here, no env var needed.
        let config = ExecutionConfig::get_execution_config_with_base(
            assignment_id,
            Some(temp_dir.path().to_str().unwrap()),
        );
        assert!(config.is_ok());

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

        let assignment_id = 999;
        let config = ExecutionConfig::get_execution_config_with_base(
            assignment_id,
            Some(temp_dir.path().to_str().unwrap()),
        );
        assert!(config.is_err());
        assert!(config.unwrap_err().contains("Failed to read config file"));
    }

    #[test]
    fn test_invalid_config_json() {
        let temp_dir = tempdir().unwrap();

        let assignment_id = 77;
        let assignment_path = temp_dir
            .path()
            .join("assignments")
            .join(assignment_id.to_string());
        fs::create_dir_all(&assignment_path).unwrap();

        let invalid_json = r#"{ "timeout_secs": "oops" }"#;
        fs::write(assignment_path.join("config.json"), invalid_json).unwrap();

        let config = ExecutionConfig::get_execution_config_with_base(
            assignment_id,
            Some(temp_dir.path().to_str().unwrap()),
        );
        assert!(config.is_err());
        assert_eq!(config.unwrap_err(), "Invalid config JSON format");
    }
}
