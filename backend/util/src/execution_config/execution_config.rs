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
#[serde(rename_all = "lowercase")]
pub enum Language {
    Cpp,
    Java,
}

#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SubmissionMode {
    Manual,
    GATLAM,
    RNG,
    CodeCoverage,
}

#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GradingPolicy {
    Best,   // highest score across submissions
    Last,   // the most recent submission
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

    #[serde(default = "default_grading_policy")]
    pub grading_policy: GradingPolicy,

    /// Maximum number of attempts (only enforced if `limit_attempts = true`).
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,

    /// If false, attempt limits are not enforced.
    #[serde(default = "default_limit_attempts")]
    pub limit_attempts: bool,

    /// Minimum percentage required to pass (0–100).
    #[serde(default = "default_pass_mark")]
    pub pass_mark: u32,

    /// If true, students may make **practice** submissions.
    /// Practice submissions never consume graded-attempt budget.
    /// Default: false
    #[serde(default = "default_allow_practice_submissions")]
    pub allow_practice_submissions: bool,
}

impl Default for MarkingOptions {
    fn default() -> Self {
        Self {
            marking_scheme: default_marking_scheme(),
            feedback_scheme: default_feedback_scheme(),
            deliminator: default_deliminator(),
            grading_policy: default_grading_policy(),
            max_attempts: default_max_attempts(),
            limit_attempts: default_limit_attempts(),
            pass_mark: default_pass_mark(),
            allow_practice_submissions: default_allow_practice_submissions()
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectSetup {
    #[serde(default = "default_language")]
    pub language: Language,
    #[serde(default = "default_submission_mode")]
    pub submission_mode: SubmissionMode,
}

impl Default for ProjectSetup {
    fn default() -> Self {
        Self {
            language: default_language(),
            submission_mode: default_submission_mode(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutionOutputOptions {
    #[serde(default = "default_stdout")]
    pub stdout: bool,
    #[serde(default)]
    pub stderr: bool,
    #[serde(default)]
    pub retcode: bool,
}

impl Default for ExecutionOutputOptions {
    fn default() -> Self {
        Self {
            stdout: true,
            stderr: false,
            retcode: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CrossoverType {
    OnePoint,
    TwoPoint,
    Uniform,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MutationType {
    BitFlip,
    Swap,
    Scramble,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeneConfig {
    pub min_value: i32,
    pub max_value: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskSpecConfig {
    #[serde(default = "default_valid_return_codes")]
    pub valid_return_codes: Vec<i32>,
    #[serde(default)]
    pub max_runtime_ms: Option<u64>,
    #[serde(default)]
    pub forbidden_outputs: Vec<String>,
}

fn default_valid_return_codes() -> Vec<i32> {
    vec![0]
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GATLAM {
    // ---- GA Config ----
    #[serde(default = "default_population_size")]
    pub population_size: usize,
    #[serde(default = "default_number_of_generations")]
    pub number_of_generations: usize,
    #[serde(default = "default_selection_size")]
    pub selection_size: usize,
    #[serde(default = "default_reproduction_probability")]
    pub reproduction_probability: f64,
    #[serde(default = "default_crossover_probability")]
    pub crossover_probability: f64,
    #[serde(default = "default_mutation_probability")]
    pub mutation_probability: f64,
    #[serde(default = "default_genes")]
    pub genes: Vec<GeneConfig>,
    #[serde(default = "default_crossover_type")]
    pub crossover_type: CrossoverType,
    #[serde(default = "default_mutation_type")]
    pub mutation_type: MutationType,

    // ---- Components ----
    #[serde(default = "default_omega1")]
    pub omega1: f64,
    #[serde(default = "default_omega2")]
    pub omega2: f64,
    #[serde(default = "default_omega3")]
    pub omega3: f64,

    // ---- TaskSpec ----
    #[serde(default)]
    pub task_spec: TaskSpecConfig,

    // ---- Optional runtime flags ----
    #[serde(default = "default_max_parallel_chromosomes")]
    pub max_parallel_chromosomes: usize,
    #[serde(default)]
    pub verbose: bool,
}

impl Default for GATLAM {
    fn default() -> Self {
        Self {
            population_size: default_population_size(),
            number_of_generations: default_number_of_generations(),
            selection_size: default_selection_size(),
            reproduction_probability: default_reproduction_probability(),
            crossover_probability: default_crossover_probability(),
            mutation_probability: default_mutation_probability(),
            genes: default_genes(),
            crossover_type: default_crossover_type(),
            mutation_type: default_mutation_type(),
            omega1: default_omega1(),
            omega2: default_omega2(),
            omega3: default_omega3(),
            task_spec: TaskSpecConfig::default(),
            max_parallel_chromosomes: default_max_parallel_chromosomes(),
            verbose: false,
        }
    }
}

impl Default for TaskSpecConfig {
    fn default() -> Self {
        Self {
            valid_return_codes: default_valid_return_codes(),
            max_runtime_ms: None,
            forbidden_outputs: vec![],
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutionConfig {
    #[serde(default)]
    pub execution: ExecutionLimits,

    #[serde(default)]
    pub marking: MarkingOptions,

    #[serde(default)]
    pub project: ProjectSetup,

    #[serde(default)]
    pub output: ExecutionOutputOptions,

    #[serde(default)]
    pub gatlam: GATLAM,
}

impl ExecutionConfig {
    pub fn default_config() -> Self {
        ExecutionConfig {
            execution: ExecutionLimits::default(),
            marking: MarkingOptions::default(),
            project: ProjectSetup::default(),
            output: ExecutionOutputOptions::default(),
            gatlam: GATLAM::default(),
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

    /// Save the configuration to disk under the derived path based on module and assignment IDs.
    pub fn save(&self, module_id: i64, assignment_id: i64) -> Result<(), String> {
        let base_path = Self::resolve_storage_root();

        let config_dir = base_path
            .join(format!("module_{}", module_id))
            .join(format!("assignment_{}", assignment_id))
            .join("config");

        // Create directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&config_dir) {
            return Err(format!("Failed to create config directory: {:?}", e));
        }

        let config_path = config_dir.join("config.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config to JSON: {}", e))?;

        fs::write(&config_path, json)
            .map_err(|e| format!("Failed to write config file to disk: {:?}", e))?;

        Ok(())
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

fn default_grading_policy() -> GradingPolicy {
    GradingPolicy::Last
}

fn default_limit_attempts() -> bool {
    false
}

fn default_max_attempts() -> u32 {
    10
}

fn default_pass_mark() -> u32 {
    50
}

fn default_allow_practice_submissions() -> bool { 
    false 
}

fn default_language() -> Language {
    Language::Cpp
}

fn default_stdout() -> bool {
    true
}

fn default_population_size() -> usize {
    100
}

fn default_number_of_generations() -> usize {
    50
}

fn default_selection_size() -> usize {
    20
}

fn default_reproduction_probability() -> f64 {
    0.8
}

fn default_crossover_probability() -> f64 {
    0.9
}

fn default_mutation_probability() -> f64 {
    0.01
}

fn default_crossover_type() -> CrossoverType {
    CrossoverType::OnePoint
}

fn default_mutation_type() -> MutationType {
    MutationType::BitFlip
}

fn default_omega1() -> f64 {
    0.5
}

fn default_omega2() -> f64 {
    0.3
}

fn default_omega3() -> f64 {
    0.2
}

fn default_max_parallel_chromosomes() -> usize {
    4
}

fn default_genes() -> Vec<GeneConfig> {
    vec![
        GeneConfig {
            min_value: -5,
            max_value: 5,
        },
        GeneConfig {
            min_value: -4,
            max_value: 9,
        },
    ]
}

fn default_submission_mode() -> SubmissionMode {
    SubmissionMode::Manual
}
