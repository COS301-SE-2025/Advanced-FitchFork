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
use crate::types::{AllocatorSchema, TaskResult};
use crate::utilities::file_loader::load_files;
use chrono::Utc;
use std::path::PathBuf;
use util::execution_config::ExecutionConfig;
use util::execution_config::MarkingScheme;

/// Represents a marking job for a single student submission.
///
/// This struct encapsulates all the input files and configuration needed to mark a submission,
/// including memo outputs, student outputs, allocation (task) schema, and optional coverage report.
///
/// # Fields
/// - `memo_outputs`: Paths to the reference (memo) output files.
/// - `student_outputs`: Paths to the student output files.
/// - `allocation_json`: Path to the JSON file describing the task/subtask structure and scoring.
/// - `coverage_report`: Optional path to a code coverage report.
/// - `comparator`: Strategy for comparing outputs (e.g., percentage, exact).
/// - `feedback`: Automated feedback generation for each subtask.
pub struct MarkingJob<'a> {
    memo_outputs: Vec<PathBuf>,
    student_outputs: Vec<PathBuf>,
    allocation_json: PathBuf,
    coverage_report: Option<PathBuf>,
    comparator: Box<dyn OutputComparator + Send + Sync + 'a>,
    feedback: Box<dyn Feedback + Send + Sync + 'a>,
    config: ExecutionConfig,
}

impl<'a> MarkingJob<'a> {
    /// Create a new marking job with required files.
    ///
    /// # Arguments
    /// * `memo_outputs` - Paths to reference (memo) output files.
    /// * `student_outputs` - Paths to student output files.
    /// * `allocation_json` - Path to the JSON file describing the marking schema.
    /// * `module_id` - ID of the module.
    /// * `assignment_id` - ID of the assignment
    pub fn new(
        memo_outputs: Vec<PathBuf>,
        student_outputs: Vec<PathBuf>,
        allocation_json: PathBuf,
        config: ExecutionConfig,
    ) -> Self {
        Self {
            memo_outputs,
            student_outputs,
            allocation_json,
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
    /// 1. Loads and validates all input files.
    /// 2. Parses allocation and coverage reports.
    /// 3. Parses memo and student outputs into tasks and subtasks.
    /// 4. Compares outputs using the configured comparator for each subtask.
    /// 5. Aggregates results and generates automated feedback.
    /// 6. Builds a detailed report with scores and feedback per task/subtask.
    pub async fn mark(self) -> Result<MarkReportResponse, MarkerError> {
        let files = load_files(
            self.memo_outputs,
            self.student_outputs,
            self.allocation_json,
            self.coverage_report,
        )?;

        let allocator: AllocatorSchema = parsers::allocator_parser::JsonAllocatorParser.parse(&files.allocator_raw, self.config.clone())?;

        let coverage_report = if let Some(ref coverage_raw) = files.coverage_raw {
            Some(parsers::coverage_parser::JsonCoverageParser.parse(coverage_raw, self.config.clone())?)
        } else {
            None
        };

        let expected_counts: Vec<usize> = allocator
            .0
            .iter()
            .filter(|t| !t.code_coverage)
            .map(|task| task.subsections.len())
            .collect();

        let submission = parsers::output_parser::OutputParser.parse(
            (
                &files.memo_contents,
                &files.student_contents,
                expected_counts,
            ),
            self.config.clone(),
        )?;

        let mut all_results: Vec<TaskResult> = Vec::new();
        let mut per_task_results: Vec<Vec<TaskResult>> = Vec::new();
        let mut per_task_subsections: Vec<Vec<crate::report::ReportSubsection>> = Vec::new();
        let mut per_task_names: Vec<String> = Vec::new();
        let mut per_task_scores: Vec<(i64, i64)> = Vec::new();

        for (_, task_entry) in allocator.0.iter().enumerate() {
            if task_entry.code_coverage {
                continue;
            }

            let submission_task = submission
                .tasks
                .iter()
                .find(|t| t.task_id.eq_ignore_ascii_case(&task_entry.id));
            let mut subsections: Vec<crate::report::ReportSubsection> = Vec::new();
            let mut task_earned = 0;
            let mut task_possible = 0;
            let mut task_results: Vec<TaskResult> = Vec::new();

            if let Some(task_output) = submission_task {
                for (sub_index, subsection) in task_entry.subsections.iter().enumerate() {
                    let student_lines = task_output
                        .student_output
                        .subtasks
                        .get(sub_index)
                        .map(|s| s.lines.clone())
                        .unwrap_or_default();

                    let memo_or_regex_lines: Vec<String> = match self.config.marking.marking_scheme {
                        MarkingScheme::Regex => subsection.regex.clone().unwrap_or_default(),
                        _ => task_output.memo_output.subtasks.get(sub_index).map(|s| s.lines.clone()).unwrap_or_default(),
                    };

                    let mut result = self.comparator.compare(subsection, &memo_or_regex_lines, &student_lines);
                    result.stderr = task_output.stderr.clone();
                    result.return_code = task_output.return_code;

                    task_earned += result.awarded;
                    task_possible += result.possible;
                    subsections.push(crate::report::ReportSubsection {
                        label: subsection.name.clone(),
                        earned: result.awarded,
                        total: result.possible,
                        feedback: String::new(),
                    });
                    task_results.push(result.clone());
                    all_results.push(result);
                }
            } else {
                let id = &task_entry.id;
                return Err(MarkerError::InputMismatch(format!(
                    "Task '{id}' from allocator not found in submission outputs"
                )));
            }

            per_task_results.push(task_results);
            per_task_subsections.push(subsections);
            per_task_names.push(task_entry.name.clone());
            per_task_scores.push((task_earned, task_possible));
        }

        let feedback_entries = self.feedback.assemble_feedback(&all_results).await?;
        let mut feedback_iter = feedback_entries.iter();

        let mut report_tasks: Vec<crate::report::ReportTask> = Vec::new();
        let mut task_counter = 1;
        let mut total_earned = 0;
        let mut total_possible = 0;
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
            total_possible += task_possible;
            task_counter += 1;
        }

        let mut coverage_total_earned: i64 = 0;
        let mut coverage_total_possible: i64 = 0;
        if coverage_report.is_some() {
            // TODO: Hard coded coverage score map for now, we can make this configurable later. These scores are what real FF uses.
            let bucket_percent: i64 = match coverage_report.clone().unwrap().coverage_percent {
                p if p < 5.0 => 0,
                p if p < 20.0 => 20,
                p if p < 40.0 => 40,
                p if p < 60.0 => 60,
                p if p < 80.0 => 80,
                _ => 100,
            };
            
            let coverage_value = allocator.0.iter().filter(|t| t.code_coverage).map(|t| t.value).sum::<i64>();
            coverage_total_earned = bucket_percent * coverage_value / 100;
            coverage_total_possible = coverage_value;
        }

        let mark = crate::report::Score {
            earned: total_earned,
            total: total_possible,
        };

        let now = Utc::now().to_rfc3339();
        let mut report = crate::report::generate_new_mark_report(now.clone(), now, report_tasks, mark);

        if coverage_report.is_some() {
            report.code_coverage = Some(crate::report::CodeCoverageReport {
                summary: Some(crate::report::Score {
                    earned: coverage_total_earned,
                    total: coverage_total_possible,
                }),
                files: coverage_report
                    .as_ref()
                    .map(|cr| {
                        cr.files
                            .iter()
                            .map(|f| crate::report::CoverageFile {
                                path: f.path.clone(),
                                earned: f.covered_lines as i64,
                                total: f.total_lines as i64,
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
            });
        }

        Ok(report.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use std::path::PathBuf;

    fn is_valid_iso8601(s: &str) -> bool {
        DateTime::parse_from_rfc3339(s).is_ok()
    }

    #[tokio::test]
    async fn test_marker_happy_path() {
        let dir = "src/test_files/marker/case1";
        let memo_outputs = vec![PathBuf::from(dir).join("memo1.txt")];
        let student_outputs = vec![PathBuf::from(dir).join("student1.txt")];
        let allocation_json = PathBuf::from(dir).join("allocator.json");

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocation_json,
            ExecutionConfig::default_config(),
        );

        let result = job.mark().await;
        assert!(result.is_ok(), "Marking should succeed");

        let response = result.unwrap();
        assert!(response.success);
        let report = &response.data;

        assert!(is_valid_iso8601(&report.created_at));
        assert!(is_valid_iso8601(&report.updated_at));

        assert_eq!(report.mark.earned, 10);
        assert_eq!(report.mark.total, 10);

        assert_eq!(report.tasks.len(), 1);
        let task = &report.tasks[0];
        assert_eq!(task.name, "Task 1");
        assert_eq!(task.score.earned, 10);
        assert_eq!(task.score.total, 10);

        assert_eq!(task.subsections.len(), 1);
        let sub = &task.subsections[0];
        assert_eq!(sub.label, "Sub1");
        assert_eq!(sub.earned, 10);
        assert_eq!(sub.total, 10);
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
        let allocation_json = PathBuf::from(dir).join("allocator.json");

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocation_json,
            ExecutionConfig::default_config(),
        );

        let result = job.mark().await;
        assert!(result.is_ok(), "Marking should succeed");

        let response = result.unwrap();
        assert!(response.success);
        let report = &response.data;

        assert!(is_valid_iso8601(&report.created_at));
        assert!(is_valid_iso8601(&report.updated_at));

        assert_eq!(report.mark.earned, 20);
        assert_eq!(report.mark.total, 30);

        assert_eq!(report.tasks.len(), 2);

        let task1 = &report.tasks[0];
        assert_eq!(task1.name, "Task 1");
        assert_eq!(task1.subsections.len(), 2);
        assert_eq!(task1.subsections[0].label, "Sub1.1");
        assert_eq!(task1.subsections[0].earned, 5);
        assert_eq!(task1.subsections[0].total, 5);
        assert!(!task1.subsections[0].feedback.is_empty());
        assert_eq!(task1.subsections[1].label, "Sub1.2");
        assert_eq!(task1.subsections[1].earned, 5);
        assert_eq!(task1.subsections[1].total, 5);
        assert!(!task1.subsections[1].feedback.is_empty());

        let task2 = &report.tasks[1];
        assert_eq!(task2.name, "Task 2");
        assert_eq!(task2.subsections.len(), 2);
        assert_eq!(task2.subsections[0].label, "Sub2.1");
        assert_eq!(task2.subsections[0].earned, 10);
        assert_eq!(task2.subsections[0].total, 10);
        assert!(!task2.subsections[0].feedback.is_empty());
        assert_eq!(task2.subsections[1].label, "Sub2.2");
        assert_eq!(task2.subsections[1].earned, 0);
        assert_eq!(task2.subsections[1].total, 10);
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
        let allocation_json = PathBuf::from(dir).join("allocator.json");

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocation_json,
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
        assert_eq!(task1.subsections[0].earned, 5);
        assert_eq!(task1.subsections[0].total, 5);
        assert!(!task1.subsections[0].feedback.is_empty());
        assert_eq!(task1.subsections[1].label, "Output Buzz");
        assert_eq!(task1.subsections[1].earned, 0);
        assert_eq!(task1.subsections[1].total, 5);
        assert!(!task1.subsections[1].feedback.is_empty());

        let task2 = &report.tasks[1];
        assert_eq!(task2.name, "Sum");
        assert_eq!(task2.subsections.len(), 2);
        assert_eq!(task2.subsections[0].label, "Sum correct");
        assert_eq!(task2.subsections[0].earned, 0);
        assert_eq!(task2.subsections[0].total, 10);
        assert!(!task2.subsections[0].feedback.is_empty());
        assert_eq!(task2.subsections[1].label, "Handles negatives");
        assert_eq!(task2.subsections[1].earned, 0);
        assert_eq!(task2.subsections[1].total, 10);
        assert!(!task2.subsections[1].feedback.is_empty());

        // Overall
        assert_eq!(report.mark.earned, 5);
        assert_eq!(report.mark.total, 30);
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
        let allocation_json = PathBuf::from(dir).join("allocator.json");

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocation_json,
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
        assert!(task1.subsections[0].earned < 5);
        assert_eq!(task1.subsections[0].total, 5);
        assert!(!task1.subsections[0].feedback.is_empty());
        // Sub2: incorrect order, expect 0
        assert_eq!(task1.subsections[1].label, "Reverse xyz");
        assert_eq!(task1.subsections[1].earned, 0);
        assert_eq!(task1.subsections[1].total, 5);
        assert!(!task1.subsections[1].feedback.is_empty());

        let task2 = &report.tasks[1];
        assert_eq!(task2.name, "Sort Numbers");
        assert_eq!(task2.subsections.len(), 2);
        // Sub1: output split across two lines, expect partial credit
        assert_eq!(task2.subsections[0].label, "Sort ascending");
        assert!(task2.subsections[0].earned < 10);
        assert_eq!(task2.subsections[0].total, 10);
        assert!(!task2.subsections[0].feedback.is_empty());
        // Sub2: out of order, expect 0
        assert_eq!(task2.subsections[1].label, "Sort descending");
        assert_eq!(task2.subsections[1].earned, 0);
        assert_eq!(task2.subsections[1].total, 10);
        assert!(!task2.subsections[1].feedback.is_empty());

        // Overall: sum of all earned points
        let total_earned = task1.subsections[0].earned
            + task1.subsections[1].earned
            + task2.subsections[0].earned
            + task2.subsections[1].earned;
        assert_eq!(report.mark.earned, total_earned);
        assert_eq!(report.mark.total, 30);
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
        let allocation_json = PathBuf::from(dir).join("allocator.json");

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocation_json,
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
        assert_eq!(task1.subsections[0].earned, 0);
        assert_eq!(task1.subsections[0].total, 5);
        assert!(!task1.subsections[0].feedback.is_empty());
        // Sub2: extra whitespace and duplicate, should be penalized
        assert_eq!(task1.subsections[1].label, "Echo World");
        assert!(task1.subsections[1].earned < 5);
        assert_eq!(task1.subsections[1].total, 5);
        assert!(!task1.subsections[1].feedback.is_empty());

        let task2 = &report.tasks[1];
        assert_eq!(task2.name, "Repeat");
        assert_eq!(task2.subsections.len(), 2);
        // Sub1: duplicate correct line, should be penalized
        assert_eq!(task2.subsections[0].label, "Repeat Yes");
        assert!(task2.subsections[0].earned < 10);
        assert_eq!(task2.subsections[0].total, 10);
        assert!(!task2.subsections[0].feedback.is_empty());
        // Sub2: missing output, should be 0
        assert_eq!(task2.subsections[1].label, "Repeat No");
        assert_eq!(task2.subsections[1].earned, 0);
        assert_eq!(task2.subsections[1].total, 10);
        assert!(!task2.subsections[1].feedback.is_empty());

        // Overall: sum of all earned points
        let total_earned = task1.subsections[0].earned
            + task1.subsections[1].earned
            + task2.subsections[0].earned
            + task2.subsections[1].earned;
        assert_eq!(report.mark.earned, total_earned);
        assert_eq!(report.mark.total, 30);
    }

    #[tokio::test]
    async fn test_marker_error_handling_missing_file() {
        let dir = "src/test_files/marker/case6";
        let memo_outputs = vec![PathBuf::from(format!("{}/memo1.txt", dir))];
        let student_outputs = vec![PathBuf::from(format!("{}/student_missing.txt", dir))];
        let allocation_json = PathBuf::from(format!("{}/allocator.json", dir));

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocation_json,
            ExecutionConfig::default_config(),
        );

        let result = job.mark().await;
        assert!(
            result.is_err(),
            "Marking should fail due to missing student file"
        );
        let err = result.unwrap_err();
        let err_str = format!("{err:?}");
        assert!(
            err_str.contains("File not found") || err_str.contains("unreadable"),
            "Error message should mention missing file, got: {err_str}"
        );
    }

    #[tokio::test]
    async fn test_marker_error_handling_invalid_allocator_json() {
        let dir = "src/test_files/marker/case7";
        let memo_outputs = vec![PathBuf::from(dir).join("memo1.txt")];
        let student_outputs = vec![PathBuf::from(dir).join("student1.txt")];
        let allocation_json = PathBuf::from(dir).join("allocator.json");

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocation_json,
            ExecutionConfig::default_config(),
        );

        let result = job.mark().await;
        assert!(
            result.is_err(),
            "Marking should fail due to invalid allocator JSON"
        );
        let err = result.unwrap_err();
        let err_str = format!("{err:?}");
        assert!(
            err_str.contains("InvalidJson")
                || err_str.contains("allocator")
                || err_str.contains("expected")
                || err_str.contains("EOF"),
            "Error message should mention invalid JSON, got: {}",
            err_str
        );
    }

    #[tokio::test]
    async fn test_marker_error_handling_empty_student_output() {
        let dir = "src/test_files/marker/case9";
        let memo_outputs = vec![PathBuf::from(dir).join("memo1.txt")];
        let student_outputs = vec![PathBuf::from(dir).join("student1.txt")];
        let allocation_json = PathBuf::from(dir).join("allocator.json");

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocation_json,
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
                assert_eq!(task.subsections[0].earned, 0);
                assert_eq!(task.subsections[1].earned, 0);
                assert_eq!(report.mark.earned, 0);
                assert_eq!(report.mark.total, 10);
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
