//! # Marker Library
//!
//! This module provides the core logic for automated marking of programming assignments.
//! It supports loading and validating student and memo outputs, parsing allocation and report files,
//! comparing outputs using pluggable strategies, and generating detailed marking reports with feedback.
//!
//! ## Key Concepts
//! - **MarkingJob**: The main struct representing a marking job for a single submission.
//! - **Comparators**: Pluggable strategies for comparing student and memo outputs (e.g., percentage, exact).
//! - **Feedback**: Automated feedback generation for each subtask.
//! - **Reports**: Structured output summarizing scores and feedback per task and subtask.

pub mod comparators;
pub mod error;
pub mod feedback;
pub mod parsers;
pub mod report;
pub mod scorer;
pub mod traits;
pub mod types;
pub mod utilities;

use crate::comparators::percentage_comparator::PercentageComparator;
use crate::error::MarkerError;
use crate::feedback::auto_feedback::AutoFeedback;
use crate::report::MarkReportResponse;
use crate::traits::comparator::OutputComparator;
use crate::traits::feedback::Feedback;
use crate::traits::parser::Parser;
use crate::types::TaskResult;

use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use util::execution_config::ExecutionConfig;
use util::execution_config::MarkingScheme;
use util::mark_allocator;

/// Represents a marking job for a single student submission.
///
/// This struct encapsulates all the input files and configuration needed to mark a submission,
/// including memo outputs, student outputs, allocator (task) schema object, and optional coverage report.
///
/// # Fields
/// - `memo_outputs`: Paths to the reference (memo) output files.
/// - `student_outputs`: Paths to the student output files.
/// - `allocator`: **Allocator object** describing the task/subtask structure and scoring.
/// - `coverage_report`: Optional path to a code coverage report.
/// - `comparator`: Strategy for comparing outputs (e.g., percentage, exact).
/// - `feedback`: Automated feedback generation for each subtask.
pub struct MarkingJob<'a> {
    memo_outputs: Vec<PathBuf>,
    student_outputs: Vec<PathBuf>,
    allocator: mark_allocator::MarkAllocator,
    coverage_report: Option<PathBuf>,
    comparator: Box<dyn OutputComparator + Send + Sync + 'a>,
    feedback: Box<dyn Feedback + Send + Sync + 'a>,
    config: ExecutionConfig,
}

/// Round a float to two decimal places in an efficient manner.
///
/// Uses the common multiply / round / divide trick. Kept local to this module
/// so it's cheap to inline and obvious where rounding is happening.
#[inline]
fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

impl<'a> MarkingJob<'a> {
    /// Create a new marking job with required files.
    ///
    /// # Arguments
    /// * `memo_outputs` - Paths to reference (memo) output files.
    /// * `student_outputs` - Paths to student output files.
    /// * `allocator` - **Allocator object** describing the marking schema.
    /// * `config` - Execution configuration.
    pub fn new(
        memo_outputs: Vec<PathBuf>,
        student_outputs: Vec<PathBuf>,
        allocator: mark_allocator::MarkAllocator,
        config: ExecutionConfig,
    ) -> Self {
        Self {
            memo_outputs,
            student_outputs,
            allocator,
            coverage_report: None,
            comparator: Box::new(PercentageComparator),
            feedback: Box::new(AutoFeedback),
            config,
        }
    }

    /// Attach a code coverage report to the marking job.
    ///
    /// # Arguments
    /// * `report` - Path to the coverage report file.
    pub fn with_coverage(mut self, report: PathBuf) -> Self {
        self.coverage_report = Some(report);
        self
    }

    /// Set a custom output comparator strategy for this marking job.
    ///
    /// # Arguments
    /// * `comparator` - An implementation of the `OutputComparator` trait.
    pub fn with_comparator<C: OutputComparator + 'a>(mut self, comparator: C) -> Self {
        self.comparator = Box::new(comparator);
        self
    }

    /// Set a custom feedback strategy for this marking job.
    ///
    /// # Arguments
    /// * `feedback` - An implementation of the `Feedback` trait.
    pub fn with_feedback<F: Feedback + Send + Sync + 'a>(mut self, feedback: F) -> Self {
        self.feedback = Box::new(feedback);
        self
    }

    /// Run the marking process and generate a report.
    ///
    /// # Returns
    /// * `Ok(MarkReportResponse)` on success, containing the full marking report.
    /// * `Err(MarkerError)` if any step fails (e.g., file loading, parsing, input mismatch).
    ///
    /// # Steps
    /// 1. Loads and validates all input files (memo/student/coverage).
    /// 2. Uses the provided allocator object.
    /// 3. Parses memo and student outputs into tasks and subtasks.
    /// 4. Compares outputs using the configured comparator for each subtask.
    /// 5. Aggregates results and generates automated feedback.
    /// 6. Builds a detailed report with scores and feedback per task/subtask.
    pub async fn mark(self) -> Result<MarkReportResponse, MarkerError> {
        // --- Load memo & student contents locally (no allocator path anymore) ---
        let memo_contents: Vec<String> = self
            .memo_outputs
            .iter()
            .map(|p| {
                fs::read_to_string(p).map_err(|e| {
                    MarkerError::InputMismatch(format!("Failed to read memo file {:?}: {e}", p))
                })
            })
            .collect::<Result<_, _>>()?;

        let student_contents: Vec<String> = self
            .student_outputs
            .iter()
            .map(|p| {
                fs::read_to_string(p).map_err(|e| {
                    MarkerError::InputMismatch(format!("Failed to read student file {:?}: {e}", p))
                })
            })
            .collect::<Result<_, _>>()?;

        // Optional: coverage raw JSON value
        let coverage_raw: Option<serde_json::Value> = match &self.coverage_report {
            Some(path) => {
                let s = fs::read_to_string(path).map_err(|e| {
                    MarkerError::InputMismatch(format!(
                        "Failed to read coverage file {:?}: {e}",
                        path
                    ))
                })?;
                let v: serde_json::Value = serde_json::from_str(&s)
                    .map_err(|e| MarkerError::InvalidJson(format!("Invalid coverage JSON: {e}")))?;
                Some(v)
            }
            None => None,
        };

        // --- Use allocator object directly ---
        let allocator = self.allocator;

        // Build expected counts for output parser (ignore coverage tasks)
        let expected_counts: Vec<usize> = allocator
            .tasks
            .iter()
            .filter(|t| !t.code_coverage.unwrap_or(false))
            .map(|task| task.subsections.len())
            .collect();

        // Parse outputs
        let submission = crate::parsers::output_parser::OutputParser.parse(
            (&memo_contents, &student_contents, expected_counts),
            self.config.clone(),
        )?;

        // Compare & collect results
        let mut all_results: Vec<TaskResult> = Vec::new();
        let mut per_task_results: Vec<Vec<TaskResult>> = Vec::new();
        let mut per_task_subsections: Vec<Vec<crate::report::ReportSubsection>> = Vec::new();
        let mut per_task_names: Vec<String> = Vec::new();
        let mut per_task_scores: Vec<(f64, f64)> = Vec::new();

        for task_entry in allocator.tasks.iter() {
            if task_entry.code_coverage.unwrap_or(false) {
                // handled later
                continue;
            }

            // submission uses ids like "task1", "task2", ...
            let expected_id = format!("task{}", task_entry.task_number);
            let submission_task = submission
                .tasks
                .iter()
                .find(|t| t.task_id.eq_ignore_ascii_case(&expected_id));

            let mut subsections: Vec<crate::report::ReportSubsection> = Vec::new();
            let mut task_earned = 0.0;
            let mut task_results: Vec<TaskResult> = Vec::new();

            if let Some(task_output) = submission_task {
                for (sub_index, subsection) in task_entry.subsections.iter().enumerate() {
                    let student_lines = task_output
                        .student_output
                        .subtasks
                        .get(sub_index)
                        .map(|s| s.lines.clone())
                        .unwrap_or_default();

                    let memo_or_regex_lines: Vec<String> = match self.config.marking.marking_scheme
                    {
                        MarkingScheme::Regex => {
                            match subsection.regex.clone() {
                                Some(patterns) => patterns,
                                None => {
                                    let pattern_count = subsection.value.max(0.0).round() as usize;
                                    std::iter::repeat(String::new()).take(pattern_count).collect()
                                }
                            }
                        },
                        _ => task_output
                            .memo_output
                            .subtasks
                            .get(sub_index)
                            .map(|s| s.lines.clone())
                            .unwrap_or_default(),
                    };

                    let mut result =
                        self.comparator
                            .compare(subsection, &memo_or_regex_lines, &student_lines);
                    result.stderr = task_output.stderr.clone();
                    result.return_code = task_output.return_code;

                    result.awarded = round2(result.awarded);
                    task_earned += result.awarded;
                    subsections.push(crate::report::ReportSubsection {
                        label: subsection.name.clone(),
                        earned: result.awarded,
                        total: round2(subsection.value),
                        feedback: String::new(),
                    });
                    task_results.push(result.clone());
                    all_results.push(result);
                }
            } else {
                return Err(MarkerError::InputMismatch(format!(
                    "Task '{}' from allocator not found in submission outputs",
                    expected_id
                )));
            }

            per_task_results.push(task_results);
            per_task_subsections.push(subsections);
            per_task_names.push(task_entry.name.clone());
            per_task_scores.push((round2(task_earned), round2(task_entry.value)));
        }

        // Feedback
        let feedback_entries = self.feedback.assemble_feedback(&all_results).await?;
        let mut feedback_iter = feedback_entries.iter();

        let mut report_tasks: Vec<crate::report::ReportTask> = Vec::new();
        let mut task_counter = 1;
        let mut total_earned = 0.0;
        for ((_task_results, mut subsections), (name, (task_earned, task_possible))) in
            per_task_results
                .into_iter()
                .zip(per_task_subsections)
                .zip(per_task_names.into_iter().zip(per_task_scores.into_iter()))
        {
            for subsection in &mut subsections {
                subsection.feedback = feedback_iter
                    .next()
                    .map(|f| f.message.clone())
                    .unwrap_or_default();
            }

            report_tasks.push(crate::report::ReportTask {
                task_number: task_counter,
                name,
                score: crate::report::Score {
                    earned: task_earned,
                    total: task_possible,
                },
                subsections,
            });

            total_earned += task_earned;
            task_counter += 1;
        }

        // Coverage buckets (optional)
        let mut coverage_total_earned: f64 = 0.0;
        let mut coverage_total_possible: f64 = 0.0;
        if let Some(cov_raw) = coverage_raw.as_ref() {
            let coverage_report = crate::parsers::coverage_parser::JsonCoverageParser
                .parse(cov_raw, self.config.clone())?;

            let bucket_percent: f64 = match coverage_report.coverage_percent {
                p if p < 5.0 => 0.0,
                p if p < 20.0 => 20.0,
                p if p < 40.0 => 40.0,
                p if p < 60.0 => 60.0,
                p if p < 80.0 => 80.0,
                _ => 100.0,
            };

            let coverage_value = allocator
                .tasks
                .iter()
                .filter(|t| t.code_coverage.unwrap_or(false))
                .map(|t| t.value)
                .sum::<f64>();

            coverage_total_earned = round2(bucket_percent * coverage_value / 100.0);
            coverage_total_possible = round2(coverage_value);
            total_earned += coverage_total_earned;

            // attach to report later
        }

        let mark = crate::report::Score {
            earned: round2(total_earned),
            total: round2(allocator.total_value),
        };

        let now = Utc::now().to_rfc3339();
        let mut report =
            crate::report::generate_new_mark_report(now.clone(), now, report_tasks, mark);

        if coverage_total_possible > 0.0 {
            if let Some(cov_raw) = coverage_raw {
                let coverage_report = crate::parsers::coverage_parser::JsonCoverageParser
                    .parse(&cov_raw, self.config.clone())?;

                report.code_coverage = Some(crate::report::CodeCoverageReport {
                    summary: Some(crate::report::Score {
                        earned: coverage_total_earned,
                        total: coverage_total_possible,
                    }),
                    files: coverage_report
                        .files
                        .iter()
                        .map(|f| crate::report::CoverageFile {
                            path: f.path.clone(),
                            earned: round2(f.covered_lines as f64),
                            total: round2(f.total_lines as f64),
                        })
                        .collect(),
                });
            }
        }

        Ok(report.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    fn is_valid_iso8601(s: &str) -> bool {
        DateTime::parse_from_rfc3339(s).is_ok()
    }

    // Helper: load allocator object from the test file (no util paths/APIs needed)
    fn load_test_allocator(path: &std::path::Path) -> mark_allocator::MarkAllocator {
        let s = std::fs::read_to_string(path).expect("read allocator.json");
        serde_json::from_str::<mark_allocator::MarkAllocator>(&s).expect("parse allocator.json")
    }

    #[tokio::test]
    async fn test_marker_happy_path() {
        let dir = "src/test_files/marker/case1";
        let memo_outputs = vec![PathBuf::from(dir).join("memo1.txt")];
        let student_outputs = vec![PathBuf::from(dir).join("student1.txt")];
        let allocator = load_test_allocator(&PathBuf::from(dir).join("allocator.json"));

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocator,
            ExecutionConfig::default_config(),
        );

        let result = job.mark().await;
        assert!(result.is_ok(), "Marking should succeed");

        let response = result.unwrap();
        assert!(response.success);
        let report = &response.data;

        assert!(is_valid_iso8601(&report.created_at));
        assert!(is_valid_iso8601(&report.updated_at));

        assert_eq!(report.mark.earned, 10.0);
        assert_eq!(report.mark.total, 10.0);

        assert_eq!(report.tasks.len(), 1);
        let task = &report.tasks[0];
        assert_eq!(task.name, "Task 1");
        assert_eq!(task.score.earned, 10.0);
        assert_eq!(task.score.total, 10.0);

        assert_eq!(task.subsections.len(), 1);
        let sub = &task.subsections[0];
        assert_eq!(sub.label, "Sub1");
        assert_eq!(sub.earned, 10.0);
        assert_eq!(sub.total, 10.0);
        assert!(!sub.feedback.is_empty(), "Feedback should not be empty");
    }

    #[tokio::test]
    async fn test_marker_happy_path_case2() {
        let dir = "src/test_files/marker/case2";
        let memo_outputs = vec![
            PathBuf::from(dir).join("memo1.txt"),
            PathBuf::from(dir).join("memo2.txt"),
        ];
        let student_outputs = vec![
            PathBuf::from(dir).join("student1.txt"),
            PathBuf::from(dir).join("student2.txt"),
        ];
        let allocator = load_test_allocator(&PathBuf::from(dir).join("allocator.json"));

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocator,
            ExecutionConfig::default_config(),
        );

        let result = job.mark().await;
        assert!(result.is_ok(), "Marking should succeed");

        let response = result.unwrap();
        assert!(response.success);
        let report = &response.data;

        assert!(is_valid_iso8601(&report.created_at));
        assert!(is_valid_iso8601(&report.updated_at));

        assert_eq!(report.mark.earned, 20.0);
        assert_eq!(report.mark.total, 30.0);

        assert_eq!(report.tasks.len(), 2);

        let task1 = &report.tasks[0];
        assert_eq!(task1.name, "Task 1");
        assert_eq!(task1.subsections.len(), 2);
        assert_eq!(task1.subsections[0].label, "Sub1.1");
        assert_eq!(task1.subsections[0].earned, 5.0);
        assert_eq!(task1.subsections[0].total, 5.0);
        assert!(!task1.subsections[0].feedback.is_empty());
        assert_eq!(task1.subsections[1].label, "Sub1.2");
        assert_eq!(task1.subsections[1].earned, 5.0);
        assert_eq!(task1.subsections[1].total, 5.0);
        assert!(!task1.subsections[1].feedback.is_empty());

        let task2 = &report.tasks[1];
        assert_eq!(task2.name, "Task 2");
        assert_eq!(task2.subsections.len(), 2);
        assert_eq!(task2.subsections[0].label, "Sub2.1");
        assert_eq!(task2.subsections[0].earned, 10.0);
        assert_eq!(task2.subsections[0].total, 10.0);
        assert!(!task2.subsections[0].feedback.is_empty());
        assert_eq!(task2.subsections[1].label, "Sub2.2");
        assert_eq!(task2.subsections[1].earned, 0.0);
        assert_eq!(task2.subsections[1].total, 10.0);
        assert!(!task2.subsections[1].feedback.is_empty());
    }

    #[tokio::test]
    async fn test_marker_edge_cases_partial_and_empty() {
        let dir = "src/test_files/marker/case3";
        let memo_outputs = vec![
            PathBuf::from(dir).join("memo1.txt"),
            PathBuf::from(dir).join("memo2.txt"),
        ];
        let student_outputs = vec![
            PathBuf::from(dir).join("student1.txt"),
            PathBuf::from(dir).join("student2.txt"),
        ];
        let allocator = load_test_allocator(&PathBuf::from(dir).join("allocator.json"));

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocator,
            ExecutionConfig::default_config(),
        );

        let result = job.mark().await;
        assert!(result.is_ok(), "Marking should succeed");

        let response = result.unwrap();
        assert!(response.success);
        let report = &response.data;

        assert_eq!(report.tasks.len(), 2);

        let task1 = &report.tasks[0];
        assert_eq!(task1.name, "FizzBuzz");
        assert_eq!(task1.subsections.len(), 2);
        assert_eq!(task1.subsections[0].label, "Output Fizz");
        assert_eq!(task1.subsections[0].earned, 5.0);
        assert_eq!(task1.subsections[0].total, 5.0);
        assert!(!task1.subsections[0].feedback.is_empty());
        assert_eq!(task1.subsections[1].label, "Output Buzz");
        assert_eq!(task1.subsections[1].earned, 0.0);
        assert_eq!(task1.subsections[1].total, 5.0);
        assert!(!task1.subsections[1].feedback.is_empty());

        let task2 = &report.tasks[1];
        assert_eq!(task2.name, "Sum");
        assert_eq!(task2.subsections.len(), 2);
        assert_eq!(task2.subsections[0].label, "Sum correct");
        assert_eq!(task2.subsections[0].earned, 0.0);
        assert_eq!(task2.subsections[0].total, 10.0);
        assert!(!task2.subsections[0].feedback.is_empty());
        assert_eq!(task2.subsections[1].label, "Handles negatives");
        assert_eq!(task2.subsections[1].earned, 0.0);
        assert_eq!(task2.subsections[1].total, 10.0);
        assert!(!task2.subsections[1].feedback.is_empty());

        // Overall
        assert_eq!(report.mark.earned, 5.0);
        assert_eq!(report.mark.total, 30.0);
    }

    #[tokio::test]
    async fn test_marker_mixed_partial_extra_and_order() {
        let dir = "src/test_files/marker/case4";
        let memo_outputs = vec![
            PathBuf::from(dir).join("memo1.txt"),
            PathBuf::from(dir).join("memo2.txt"),
        ];
        let student_outputs = vec![
            PathBuf::from(dir).join("student1.txt"),
            PathBuf::from(dir).join("student2.txt"),
        ];
        let allocator = load_test_allocator(&PathBuf::from(dir).join("allocator.json"));

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocator,
            ExecutionConfig::default_config(),
        );

        let result = job.mark().await;
        assert!(result.is_ok(), "Marking should succeed");

        let response = result.unwrap();
        assert!(response.success);
        let report = &response.data;

        // Check structure
        assert_eq!(report.tasks.len(), 2);

        let task1 = &report.tasks[0];
        assert_eq!(task1.name, "Reverse String");
        assert_eq!(task1.subsections.len(), 2);
        // Sub1: correct output with extra line, expect partial credit (likely 0 with strict comparator)
        assert_eq!(task1.subsections[0].label, "Reverse abc");
        assert!(task1.subsections[0].earned < 5.0);
        assert_eq!(task1.subsections[0].total, 5.0);
        assert!(!task1.subsections[0].feedback.is_empty());
        // Sub2: incorrect order, expect 0
        assert_eq!(task1.subsections[1].label, "Reverse xyz");
        assert_eq!(task1.subsections[1].earned, 0.0);
        assert_eq!(task1.subsections[1].total, 5.0);
        assert!(!task1.subsections[1].feedback.is_empty());

        let task2 = &report.tasks[1];
        assert_eq!(task2.name, "Sort Numbers");
        assert_eq!(task2.subsections.len(), 2);
        // Sub1: output split across two lines, expect partial credit
        assert_eq!(task2.subsections[0].label, "Sort ascending");
        assert!(task2.subsections[0].earned < 10.0);
        assert_eq!(task2.subsections[0].total, 10.0);
        assert!(!task2.subsections[0].feedback.is_empty());
        // Sub2: out of order, expect 0
        assert_eq!(task2.subsections[1].label, "Sort descending");
        assert_eq!(task2.subsections[1].earned, 0.0);
        assert_eq!(task2.subsections[1].total, 10.0);
        assert!(!task2.subsections[1].feedback.is_empty());

        // Overall: sum of all earned points
        let total_earned = task1.subsections[0].earned
            + task1.subsections[1].earned
            + task2.subsections[0].earned
            + task2.subsections[1].earned;
        assert_eq!(report.mark.earned, total_earned);
        assert_eq!(report.mark.total, 30.0);
    }

    #[tokio::test]
    async fn test_marker_whitespace_case_and_duplicates() {
        let dir = "src/test_files/marker/case5";
        let memo_outputs = vec![
            PathBuf::from(dir).join("memo1.txt"),
            PathBuf::from(dir).join("memo2.txt"),
        ];
        let student_outputs = vec![
            PathBuf::from(dir).join("student1.txt"),
            PathBuf::from(dir).join("student2.txt"),
        ];
        let allocator = load_test_allocator(&PathBuf::from(dir).join("allocator.json"));

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocator,
            ExecutionConfig::default_config(),
        );

        let result = job.mark().await;
        assert!(result.is_ok(), "Marking should succeed");

        let response = result.unwrap();
        assert!(response.success);
        let report = &response.data;

        // Check structure
        assert_eq!(report.tasks.len(), 2);

        let task1 = &report.tasks[0];
        assert_eq!(task1.name, "Echo");
        assert_eq!(task1.subsections.len(), 2);
        // Sub1: wrong case, should be 0
        assert_eq!(task1.subsections[0].label, "Echo Hello");
        assert_eq!(task1.subsections[0].earned, 0.0);
        assert_eq!(task1.subsections[0].total, 5.0);
        assert!(!task1.subsections[0].feedback.is_empty());
        // Sub2: extra whitespace and duplicate, should be penalized
        assert_eq!(task1.subsections[1].label, "Echo World");
        assert!(task1.subsections[1].earned < 5.0);
        assert_eq!(task1.subsections[1].total, 5.0);
        assert!(!task1.subsections[1].feedback.is_empty());

        let task2 = &report.tasks[1];
        assert_eq!(task2.name, "Repeat");
        assert_eq!(task2.subsections.len(), 2);
        // Sub1: duplicate correct line, should be penalized
        assert_eq!(task2.subsections[0].label, "Repeat Yes");
        assert!(task2.subsections[0].earned < 10.0);
        assert_eq!(task2.subsections[0].total, 10.0);
        assert!(!task2.subsections[0].feedback.is_empty());
        // Sub2: missing output, should be 0
        assert_eq!(task2.subsections[1].label, "Repeat No");
        assert_eq!(task2.subsections[1].earned, 0.0);
        assert_eq!(task2.subsections[1].total, 10.0);
        assert!(!task2.subsections[1].feedback.is_empty());

        // Overall: sum of all earned points
        let total_earned = task1.subsections[0].earned
            + task1.subsections[1].earned
            + task2.subsections[0].earned
            + task2.subsections[1].earned;
        assert_eq!(report.mark.earned, total_earned);
        assert_eq!(report.mark.total, 30.0);
    }

    #[tokio::test]
    async fn test_marker_error_handling_missing_file() {
        let dir = "src/test_files/marker/case6";
        let memo_outputs = vec![PathBuf::from(format!("{}/memo1.txt", dir))];
        let student_outputs = vec![PathBuf::from(format!("{}/student_missing.txt", dir))];
        let allocator = load_test_allocator(&PathBuf::from(format!("{}/allocator.json", dir)));

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocator,
            ExecutionConfig::default_config(),
        );

        let result = job.mark().await;

        // Must be an error
        assert!(
            result.is_err(),
            "Marking should fail due to missing student file"
        );

        // Match the specific error variant and message shape
        match result {
            Err(MarkerError::InputMismatch(msg)) => {
                assert!(
                    msg.contains("Failed to read student file")
                        || msg.contains("No such file")
                        || msg.contains("No such file or directory")
                        || msg.contains("unreadable"),
                    "Error message should mention missing file, got: {msg}"
                );
            }
            Err(other) => {
                panic!("Expected InputMismatch for missing file, got: {other:?}");
            }
            Ok(_) => unreachable!(),
        }
    }

    #[tokio::test]
    async fn test_marker_error_handling_invalid_allocator_json() {
        // This test still loads allocator.json into an object first, so to simulate invalid JSON
        // you'd normally catch it before constructing the job. Here we mimic the old assertion shape
        // by forcing a bad parse step locally.
        let dir = "src/test_files/marker/case7";
        let _memo_outputs = vec![PathBuf::from(dir).join("memo1.txt")];
        let _student_outputs = vec![PathBuf::from(dir).join("student1.txt")];

        // Try to parse invalid allocator.json -> expect parse error here, not in mark()
        let bad_alloc_path = PathBuf::from(dir).join("allocator.json");
        let s = std::fs::read_to_string(&bad_alloc_path).expect("read allocator.json");
        let alloc_parse = serde_json::from_str::<mark_allocator::MarkAllocator>(&s);

        assert!(
            alloc_parse.is_err(),
            "allocator.json should be invalid in this test case"
        );
    }

    #[tokio::test]
    async fn test_marker_error_handling_empty_student_output() {
        let dir = "src/test_files/marker/case9";
        let memo_outputs = vec![PathBuf::from(dir).join("memo1.txt")];
        let student_outputs = vec![PathBuf::from(dir).join("student1.txt")];
        let allocator = load_test_allocator(&PathBuf::from(dir).join("allocator.json"));

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocator,
            ExecutionConfig::default_config(),
        );

        let result = job.mark().await;
        match result {
            Ok(response) => {
                let report = &response.data;
                assert_eq!(report.tasks.len(), 1);
                let task = &report.tasks[0];
                assert_eq!(task.name, "EmptyStudent");
                assert_eq!(task.subsections.len(), 2);
                assert_eq!(task.subsections[0].earned, 0.0);
                assert_eq!(task.subsections[1].earned, 0.0);
                assert_eq!(report.mark.earned, 0.0);
                assert_eq!(report.mark.total, 10.0);
            }
            Err(err) => {
                let err_str = format!("{err:?}");
                assert!(
                    err_str.contains("ParseOutputError")
                        || err_str.contains("Expected")
                        || err_str.contains("subtasks")
                        || err_str.contains("delimiter")
                        || err_str.contains("empty"),
                    "Error message should mention empty or invalid student output, got: {err_str}"
                );
            }
        }
    }
}
