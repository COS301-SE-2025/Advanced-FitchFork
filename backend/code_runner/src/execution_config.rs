use serde::Deserialize;
use std::{env, fs, path::PathBuf};

#[derive(Debug, Clone, Deserialize)]

/// Configuration for resource-limited code execution environments.
///
/// This struct is loaded from a JSON file per assignment, and passed to Docker
/// to enforce memory, CPU, timeout, and process constraints.
pub struct ExecutionConfig {
    /// Maximum execution time (in seconds) for the container.
    pub timeout_secs: u64,

    /// Maximum memory allocated to the container (e.g., "128m").
    pub max_memory: String,

    /// Maximum number of CPU cores the container may use (e.g., "1.0").
    pub max_cpus: String,

    /// Maximum total size of decompressed files from zip archives.
    pub max_uncompressed_size: u64,

    /// Maximum number of processes allowed within the container.
    pub max_processes: u32,

    /// Programming language identifier (used to choose execution method).
    pub language: String,
}

/// Resolves the storage root directory from the `ASSIGNMENT_STORAGE_ROOT` environment variable,
/// or falls back to a default path relative to the backend root.
///
/// Ensures the path is absolute for consistent access in tests and Docker builds.
impl ExecutionConfig {
    /// Resolves the storage root directory from the `ASSIGNMENT_STORAGE_ROOT` environment variable,
    /// or falls back to a default path relative to the backend root.
    ///
    /// Ensures the path is absolute for consistent access in tests and Docker builds.
    fn resolve_storage_root() -> PathBuf {
        if let Ok(p) = env::var("ASSIGNMENT_STORAGE_ROOT") {
            let path = PathBuf::from(p);
            if path.is_relative() {
                let mut adjusted = env::current_dir().unwrap();
                adjusted.pop();
                adjusted.push(path);
                adjusted
            } else {
                path
            }
        } else {
            PathBuf::from("../data/assignment_files") // fallback with correct relative path
        }
    }

    /// Loads an execution config from a JSON file located at:
    /// `<base_path>/module_<module_id>/assignment_<assignment_id>/config/*.json`
    ///
    /// If `base_path` is not provided, uses the default resolved storage root.
    ///
    /// Returns `Err` if the config directory cannot be read, no config file is found,
    /// or the JSON is invalid.
    pub fn get_execution_config_with_base(
        module_id: i64,
        assignment_id: i64,
        base_path: Option<&str>,
    ) -> Result<Self, String> {
        let base_path = base_path
            .map(PathBuf::from)
            .unwrap_or_else(Self::resolve_storage_root);

        let config_dir = base_path
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id))
            .join("config");

        let entries = fs::read_dir(&config_dir)
            .map_err(|_| format!("Failed to read config dir at {:?}", config_dir))?;

        let mut config_file_path = None;
        for entry in entries {
            let entry = entry.map_err(|_| "Failed to read config dir entry")?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                config_file_path = Some(path);
                break;
            }
        }

        let config_path = config_file_path
            .ok_or_else(|| format!("No config json file found in config dir {:?}", config_dir))?;

        let file_contents = fs::read_to_string(&config_path)
            .map_err(|_| format!("Failed to read config file at {:?}", config_path))?;

        serde_json::from_str(&file_contents).map_err(|_| "Invalid config JSON format".to_string())
    }

    pub fn get_execution_config(module_id: i64, assignment_id: i64) -> Result<Self, String> {
        Self::get_execution_config_with_base(module_id, assignment_id, None)
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

        let module_id = 123;
        let assignment_id = 42;

        let config_dir = temp_dir
            .path()
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id))
            .join("config");
        fs::create_dir_all(&config_dir).unwrap();

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

        let config_path = config_dir.join("124.json");
        fs::write(&config_path, config_json).unwrap();

        let config = ExecutionConfig::get_execution_config_with_base(
            module_id,
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

        let module_id = 123;
        let assignment_id = 999;

        let config_dir = temp_dir
            .path()
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id))
            .join("config");
        fs::create_dir_all(&config_dir).unwrap();

        let config = ExecutionConfig::get_execution_config_with_base(
            module_id,
            assignment_id,
            Some(temp_dir.path().to_str().unwrap()),
        );

        assert!(config.is_err());
        assert!(config.unwrap_err().contains("No config json file found"));
    }

    #[test]
    fn test_invalid_config_json() {
        let temp_dir = tempdir().unwrap();

        let module_id = 123;
        let assignment_id = 77;

        let config_dir = temp_dir
            .path()
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id))
            .join("config");
        fs::create_dir_all(&config_dir).unwrap();

        let invalid_json = r#"{ "timeout_secs": "oops" }"#;
        fs::write(config_dir.join("config.json"), invalid_json).unwrap();

        let config = ExecutionConfig::get_execution_config_with_base(
            module_id,
            assignment_id,
            Some(temp_dir.path().to_str().unwrap()),
        );

        assert!(config.is_err());
        assert_eq!(config.unwrap_err(), "Invalid config JSON format");
    }
}
