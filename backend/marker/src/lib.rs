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

pub mod scorer;
pub mod feedback;
pub mod report;
pub mod error;
pub mod traits;
pub mod parsers;
pub mod utilities;
pub mod types;
pub mod comparators;

use std::path::PathBuf;
use crate::comparators::percentage_comparator::PercentageComparator;
use crate::feedback::auto_feedback::AutoFeedback;
use crate::error::MarkerError;
use crate::traits::parser::Parser;
use crate::utilities::file_loader::load_files;
use crate::traits::feedback::Feedback;
use crate::traits::comparator::OutputComparator;
use crate::report::MarkReportResponse;
use crate::types::{AllocatorSchema, TaskResult};
use crate::parsers::coverage_parser::CoverageReport;
use crate::parsers::complexity_parser::ComplexityReport;
use chrono::Utc;

/// Represents a marking job for a single student submission.
///
/// This struct encapsulates all the input files and configuration needed to mark a submission,
/// including memo outputs, student outputs, allocation (task) schema, and optional coverage/complexity reports.
///
/// # Fields
/// - `memo_outputs`: Paths to the reference (memo) output files.
/// - `student_outputs`: Paths to the student output files.
/// - `allocation_json`: Path to the JSON file describing the task/subtask structure and scoring.
/// - `coverage_report`: Optional path to a code coverage report.
/// - `complexity_report`: Optional path to a code complexity report.
/// - `comparator`: Strategy for comparing outputs (e.g., percentage, exact).
/// - `feedback`: Automated feedback generation for each subtask.
pub struct MarkingJob<'a> {
    memo_outputs: Vec<PathBuf>,
    student_outputs: Vec<PathBuf>,
    allocation_json: PathBuf,
    coverage_report: Option<PathBuf>,
    complexity_report: Option<PathBuf>,
    comparator: Box<dyn OutputComparator + Send + Sync + 'a>,
    feedback: Box<dyn Feedback + Send + Sync + 'a>,
}

impl<'a> MarkingJob<'a> {
    /// Create a new marking job with required files.
    ///
    /// # Arguments
    /// * `memo_outputs` - Paths to reference (memo) output files.
    /// * `student_outputs` - Paths to student output files.
    /// * `allocation_json` - Path to the JSON file describing the marking schema.
    pub fn new(
        memo_outputs: Vec<PathBuf>,
        student_outputs: Vec<PathBuf>,
        allocation_json: PathBuf,
    ) -> Self {
        Self {
            memo_outputs,
            student_outputs,
            allocation_json,
            coverage_report: None,
            complexity_report: None,
            comparator: Box::new(PercentageComparator),
            feedback: Box::new(AutoFeedback),
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

    /// Attach a code complexity report to the marking job.
    ///
    /// # Arguments
    /// * `report` - Path to the complexity report file.
    pub fn with_complexity(mut self, report: PathBuf) -> Self {
        self.complexity_report = Some(report);
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
    /// 2. Parses allocation, coverage, and complexity reports.
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
            self.complexity_report,
        )?;

        let allocator: AllocatorSchema = parsers::allocator_parser::JsonAllocatorParser.parse(&files.allocator_raw)?;

        if let Some(coverage_raw) = files.coverage_raw {
            let _coverage: CoverageReport = parsers::coverage_parser::JsonCoverageParser.parse(&coverage_raw)?;
        }
        
        if let Some(complexity_raw) = files.complexity_raw {
            let _complexity: ComplexityReport = parsers::complexity_parser::JsonComplexityParser.parse(&complexity_raw)?;
        }

        let submission = parsers::output_parser::OutputParser.parse((
            &files.memo_contents,
            &files.student_contents,
            allocator
                .0
                .iter()
                .map(|task| task.subsections.len())
                .collect(),
        ))?;

        let mut all_results: Vec<TaskResult> = Vec::new();
        let mut per_task_results: Vec<Vec<TaskResult>> = Vec::new();
        let mut per_task_subsections: Vec<Vec<crate::report::ReportSubsection>> = Vec::new();
        let mut per_task_names: Vec<String> = Vec::new();
        let mut per_task_scores: Vec<(u32, u32)> = Vec::new();

        for task_entry in &allocator.0 {
            let submission_task = submission.tasks.iter().find(|t| t.task_id.eq_ignore_ascii_case(&task_entry.id));
            let mut subsections: Vec<crate::report::ReportSubsection> = Vec::new();
            let mut task_earned = 0u32;
            let mut task_possible = 0u32;
            let mut task_results: Vec<TaskResult> = Vec::new();

            if let Some(task_output) = submission_task {
                for (i, subsection) in task_entry.subsections.iter().enumerate() {
                    if i >= task_output.memo_output.subtasks.len() {
                        return Err(MarkerError::InputMismatch(format!(
                            "Task '{}' has more subsections in allocator than in memo output",
                            task_entry.id
                        )));
                    }

                    let memo_lines = &task_output.memo_output.subtasks[i].lines;
                    let student_lines = &task_output.student_output.subtasks[i].lines;

                    let result: TaskResult = self.comparator.compare(
                        subsection,
                        memo_lines,
                        student_lines,
                    );

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
                return Err(MarkerError::InputMismatch(format!(
                    "Task '{}' from allocator not found in submission outputs",
                    task_entry.id
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
        let mut task_counter = 1u32;
        let mut total_earned = 0u32;
        let mut total_possible = 0u32;
        for ((_task_results, mut subsections), (name, (task_earned, task_possible))) in per_task_results.into_iter().zip(per_task_subsections).zip(per_task_names.into_iter().zip(per_task_scores.into_iter())) {
            for subsection in &mut subsections {
                subsection.feedback = feedback_iter.next().map(|f| f.message.clone()).unwrap_or_default();
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

        let mark = crate::report::Score {
            earned: total_earned,
            total: total_possible,
        };

        let now = Utc::now().to_rfc3339();
        let report = crate::report::generate_new_mark_report(
            now.clone(),
            now,
            report_tasks,
            mark,
        );

        Ok(report.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use chrono::DateTime;

    fn is_valid_iso8601(s: &str) -> bool {
        DateTime::parse_from_rfc3339(s).is_ok()
    }

    #[tokio::test]
    async fn test_marker_happy_path() {
        let dir = "src/test_files/marker/case1";
        let memo_outputs = vec![PathBuf::from(format!("{}/memo1.txt", dir))];
        let student_outputs = vec![PathBuf::from(format!("{}/student1.txt", dir))];
        let allocation_json = PathBuf::from(format!("{}/allocator.json", dir));

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocation_json,
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
            PathBuf::from(format!("{}/memo1.txt", dir)),
            PathBuf::from(format!("{}/memo2.txt", dir)),
        ];
        let student_outputs = vec![
            PathBuf::from(format!("{}/student1.txt", dir)),
            PathBuf::from(format!("{}/student2.txt", dir)),
        ];
        let allocation_json = PathBuf::from(format!("{}/allocator.json", dir));

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocation_json,
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
        assert_eq!(task1.score.earned, 10);
        assert_eq!(task1.score.total, 10);
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
        assert_eq!(task2.score.earned, 10);
        assert_eq!(task2.score.total, 20);
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
            PathBuf::from(format!("{}/memo1.txt", dir)),
            PathBuf::from(format!("{}/memo2.txt", dir)),
        ];
        let student_outputs = vec![
            PathBuf::from(format!("{}/student1.txt", dir)),
            PathBuf::from(format!("{}/student2.txt", dir)),
        ];
        let allocation_json = PathBuf::from(format!("{}/allocator.json", dir));

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocation_json,
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
            PathBuf::from(format!("{}/memo1.txt", dir)),
            PathBuf::from(format!("{}/memo2.txt", dir)),
        ];
        let student_outputs = vec![
            PathBuf::from(format!("{}/student1.txt", dir)),
            PathBuf::from(format!("{}/student2.txt", dir)),
        ];
        let allocation_json = PathBuf::from(format!("{}/allocator.json", dir));

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocation_json,
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
            PathBuf::from(format!("{}/memo1.txt", dir)),
            PathBuf::from(format!("{}/memo2.txt", dir)),
        ];
        let student_outputs = vec![
            PathBuf::from(format!("{}/student1.txt", dir)),
            PathBuf::from(format!("{}/student2.txt", dir)),
        ];
        let allocation_json = PathBuf::from(format!("{}/allocator.json", dir));

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocation_json,
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
    async fn test_marker_error_handling_mismatched_subsections() {
        let dir = "src/test_files/marker/case6";
        let memo_outputs = vec![
            PathBuf::from(format!("{}/memo1.txt", dir)),
        ];
        let student_outputs = vec![
            PathBuf::from(format!("{}/student1.txt", dir)),
        ];
        let allocation_json = PathBuf::from(format!("{}/allocator.json", dir));

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocation_json,
        );

        let result = job.mark().await;
        assert!(result.is_err(), "Marking should fail due to mismatched subsection count");
        let err = result.unwrap_err();
        let err_str = format!("{:?}", err);
        assert!(
            err_str.contains("more subsections in allocator than in memo output")
            || err_str.contains("InputMismatch")
            || err_str.contains("Expected 2 subtasks, but found 1 delimiters"),
            "Error message should mention mismatched subsections, got: {}", err_str
        );
    }

    #[tokio::test]
    async fn test_marker_error_handling_missing_file() {
        let dir = "src/test_files/marker/case6";
        let memo_outputs = vec![
            PathBuf::from(format!("{}/memo1.txt", dir)),
        ];
        // Purposely reference a missing student file
        let student_outputs = vec![
            PathBuf::from(format!("{}/student_missing.txt", dir)),
        ];
        let allocation_json = PathBuf::from(format!("{}/allocator.json", dir));

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocation_json,
        );

        let result = job.mark().await;
        assert!(result.is_err(), "Marking should fail due to missing student file");
        let err = result.unwrap_err();
        let err_str = format!("{:?}", err);
        assert!(
            err_str.contains("File not found")
            || err_str.contains("unreadable"),
            "Error message should mention missing file, got: {}", err_str
        );
    }

    #[tokio::test]
    async fn test_marker_error_handling_invalid_allocator_json() {
        let dir = "src/test_files/marker/case7";
        let memo_outputs = vec![
            PathBuf::from(format!("{}/memo1.txt", dir)),
        ];
        let student_outputs = vec![
            PathBuf::from(format!("{}/student1.txt", dir)),
        ];
        let allocation_json = PathBuf::from(format!("{}/allocator.json", dir));

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocation_json,
        );

        let result = job.mark().await;
        assert!(result.is_err(), "Marking should fail due to invalid allocator JSON");
        let err = result.unwrap_err();
        let err_str = format!("{:?}", err);
        assert!(
            err_str.contains("InvalidJson")
            || err_str.contains("allocator")
            || err_str.contains("expected")
            || err_str.contains("EOF"),
            "Error message should mention invalid JSON, got: {}", err_str
        );
    }

    #[tokio::test]
    async fn test_marker_error_handling_invalid_memo_format() {
        let dir = "src/test_files/marker/case8";
        let memo_outputs = vec![
            PathBuf::from(format!("{}/memo1.txt", dir)),
        ];
        let student_outputs = vec![
            PathBuf::from(format!("{}/student1.txt", dir)),
        ];
        let allocation_json = PathBuf::from(format!("{}/allocator.json", dir));

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocation_json,
        );

        let result = job.mark().await;
        assert!(result.is_err(), "Marking should fail due to invalid memo output format");
        let err = result.unwrap_err();
        let err_str = format!("{:?}", err);
        assert!(
            err_str.contains("ParseOutputError")
            || err_str.contains("Expected")
            || err_str.contains("subtasks")
            || err_str.contains("delimiter"),
            "Error message should mention invalid memo format, got: {}", err_str
        );
    }

    #[tokio::test]
    async fn test_marker_error_handling_empty_student_output() {
        let dir = "src/test_files/marker/case9";
        let memo_outputs = vec![
            PathBuf::from(format!("{}/memo1.txt", dir)),
        ];
        let student_outputs = vec![
            PathBuf::from(format!("{}/student1.txt", dir)),
        ];
        let allocation_json = PathBuf::from(format!("{}/allocator.json", dir));

        let job = MarkingJob::new(
            memo_outputs,
            student_outputs,
            allocation_json,
        );

        let result = job.mark().await;
        // The marker may either return Ok with 0 marks, or an error if it expects at least one delimiter.
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
                let err_str = format!("{:?}", err);
                assert!(
                    err_str.contains("ParseOutputError")
                    || err_str.contains("Expected")
                    || err_str.contains("subtasks")
                    || err_str.contains("delimiter")
                    || err_str.contains("empty"),
                    "Error message should mention empty or invalid student output, got: {}", err_str
                );
            }
        }
    }
}