use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MarkingScheme {
    Exact,
    Percentage,
    Regex,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FeedbackScheme {
    Auto,
    Manual,
    Ai,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutionLimits {
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,

    #[serde(default = "default_max_memory")]
    pub max_memory: u64,

    #[serde(default = "default_max_cpus")]
    pub max_cpus: u32,

    #[serde(default = "default_max_uncompressed_size")]
    pub max_uncompressed_size: u64,

    #[serde(default = "default_max_processes")]
    pub max_processes: u32,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            timeout_secs: default_timeout_secs(),
            max_memory: default_max_memory(),
            max_cpus: default_max_cpus(),
            max_uncompressed_size: default_max_uncompressed_size(),
            max_processes: default_max_processes(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarkingOptions {
    #[serde(default = "default_marking_scheme")]
    pub marking_scheme: MarkingScheme,

    #[serde(default = "default_feedback_scheme")]
    pub feedback_scheme: FeedbackScheme,

    #[serde(default = "default_deliminator")]
    pub deliminator: String,
}

impl Default for MarkingOptions {
    fn default() -> Self {
        Self {
            marking_scheme: default_marking_scheme(),
            feedback_scheme: default_feedback_scheme(),
            deliminator: default_deliminator(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutionConfig {
    #[serde(default)]
    pub execution: ExecutionLimits,

    #[serde(default)]
    pub marking: MarkingOptions,
}

impl ExecutionConfig {
    pub fn default_config() -> Self {
        ExecutionConfig {
            execution: ExecutionLimits::default(),
            marking: MarkingOptions::default(),
        }
    }

    fn resolve_storage_root() -> PathBuf {
        if let Ok(p) = env::var("ASSIGNMENT_STORAGE_ROOT") {
            let path = PathBuf::from(p);
            if path.is_relative() {
                let mut adjusted = env::current_dir().expect("failed to get current dir");

                if !cfg!(windows) {
                    adjusted.pop();
                }

                adjusted.push(path);
                adjusted
            } else {
                path
            }
        } else {
            PathBuf::from("../data/assignment_files")
        }
    }

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

//Default Functions

fn default_timeout_secs() -> u64 {
    10
}

fn default_max_memory() -> u64 {
    8_589_934_592
}

fn default_max_cpus() -> u32 {
    2
}

fn default_max_uncompressed_size() -> u64 {
    100_000_000
}

fn default_max_processes() -> u32 {
    256
}

fn default_marking_scheme() -> MarkingScheme {
    MarkingScheme::Exact
}

fn default_feedback_scheme() -> FeedbackScheme {
    FeedbackScheme::Auto
}

fn default_deliminator() -> String {
    "&-=-&".to_string()
}
