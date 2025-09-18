//! # Report Module
//!
//! This module defines the data structures and utilities for representing grading reports in the FitchFork system.
//! It provides a comprehensive model for scores, tasks, subtasks, code coverage, as well as
//! serialization for API responses. The main entry point is the `MarkReport` struct, which aggregates all relevant
//! grading information for a submission.
//!
//! ## Main Components
//! - [`Score`]: Represents a simple earned/total score.
//! - [`ReportSubsection`]: Represents a subtask or subcomponent of a grading task, with feedback.
//! - [`ReportTask`]: Represents a grading task, which may have multiple subsections.
//! - [`CodeCoverageReport`]: Represents code coverage results, including per-file details.
//! - [`MarkReport`]: The top-level report, aggregating all grading information.
//! - [`MarkReportResponse`]: API response wrapper for a grading report.
//! - [`generate_new_mark_report`]: Utility function to create a new `MarkReport` with default optional fields.
//!
//! ## Usage
//! These types are used throughout the marker service to construct, serialize, and return grading results for assignments.
//! They are designed to be extensible and to support optional code coverage reporting.
//!
//! ## Example JSON Output
//!
//! ```json
//! {
//!   "id": 42,
//!   "created_at": "2024-06-01T12:00:00Z",
//!   "updated_at": "2024-06-01T12:00:00Z",
//!   "mark": { "earned": 18, "total": 20 },
//!   "tasks": [
//!     {
//!       "task_number": 1,
//!       "name": "Task 1",
//!       "score": { "earned": 8, "total": 10 },
//!       "subsections": [
//!         {
//!           "label": "Subtask 1",
//!           "earned": 4,
//!           "total": 5,
//!           "feedback": "Good job"
//!         },
//!         {
//!           "label": "Subtask 2",
//!           "earned": 4,
//!           "total": 5,
//!           "feedback": "Needs improvement"
//!         }
//!       ]
//!     },
//!     {
//!       "task_number": 2,
//!       "name": "Task 2",
//!       "score": { "earned": 10, "total": 10 },
//!       "subsections": []
//!     }
//!   ],
//!   "code_coverage": {
//!     "summary": { "earned": 9, "total": 10 },
//!     "files": [
//!       { "path": "src/lib.rs", "earned": 5, "total": 5 },
//!       { "path": "src/main.rs", "earned": 4, "total": 5 }
//!     ]
//!   },
//! }
//! ```
//!

use serde::Serialize;

/// Represents a simple score with earned and total points.
#[derive(Debug, Serialize, Clone)]
pub struct Score {
    /// Points earned by the student.
    pub earned: i64,
    /// Total possible points.
    pub total: i64,
}

/// Represents a subsection of a grading task, such as a subtask or rubric item.
#[derive(Debug, Serialize, Clone)]
pub struct ReportSubsection {
    /// Label or name of the subsection (e.g., "Subtask 1").
    pub label: String,
    /// Points earned for this subsection.
    pub earned:i64,
    /// Total possible points for this subsection.
    pub total: i64,
    /// Feedback or comments for this subsection.
    pub feedback: String,
}

/// Represents a grading task, which may have multiple subsections.
#[derive(Debug, Serialize, Clone)]
pub struct ReportTask {
    /// Task number (e.g., 1 for the first task).
    pub task_number: i64,
    /// Name or description of the task.
    pub name: String,
    /// Score for the task.
    pub score: Score,
    /// Subsections (subtasks or rubric items) for this task.
    pub subsections: Vec<ReportSubsection>,
}

/// Represents a code coverage report, including a summary and per-file details.
#[derive(Debug, Serialize, Clone)]
pub struct CodeCoverageReport {
    /// Optional summary score for code coverage.
    pub summary: Option<Score>,
    /// Coverage details for each file.
    pub files: Vec<CoverageFile>,
}

/// Represents code coverage information for a single file.
#[derive(Debug, Serialize, Clone)]
pub struct CoverageFile {
    /// File path (relative or absolute).
    pub path: String,
    /// Lines or items covered in this file.
    pub earned: i64,
    /// Total lines or items in this file.
    pub total: i64,
}

/// The top-level grading report, aggregating all tasks, scores, and optional code analysis results.
#[derive(Debug, Serialize, Clone)]
pub struct MarkReport {
    /// ISO 8601 timestamp for when the report was created.
    pub created_at: String,
    /// ISO 8601 timestamp for when the report was last updated.
    pub updated_at: String,
    /// Overall mark (score) for the submission.
    pub mark: Score,
    /// List of grading tasks and their results.
    pub tasks: Vec<ReportTask>,
    /// Optional code coverage report.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_coverage: Option<CodeCoverageReport>,
}

/// API response wrapper for a grading report, used for serialization.
#[derive(Debug, Serialize)]
pub struct MarkReportResponse {
    /// Indicates if the grading was successful.
    pub success: bool,
    /// Human-readable message about the grading result.
    pub message: String,
    /// The actual grading report data.
    pub data: MarkReport,
}

impl From<MarkReport> for MarkReportResponse {
    /// Converts a `MarkReport` into a `MarkReportResponse` with a default success message.
    fn from(report: MarkReport) -> Self {
        MarkReportResponse {
            success: true,
            message: "Grading complete.".to_string(),
            data: report,
        }
    }
}

/// Utility function to create a new `MarkReport` with default `None` for optional fields.
///
/// # Arguments
/// * `created_at` - Creation timestamp (ISO 8601).
/// * `updated_at` - Update timestamp (ISO 8601).
/// * `tasks` - List of grading tasks.
/// * `mark` - Overall score.
///
/// # Returns
/// A new `MarkReport` instance with `code_coverage` set to `None`.
pub fn generate_new_mark_report(
    created_at: String,
    updated_at: String,
    tasks: Vec<ReportTask>,
    mark: Score,
) -> MarkReport {
    MarkReport {
        created_at,
        updated_at,
        mark,
        tasks,
        code_coverage: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    fn sample_score() -> Score {
        Score { earned: 8, total: 10 }
    }

    fn sample_subsection() -> ReportSubsection {
        ReportSubsection {
            label: "Subtask 1".to_string(),
            earned: 4,
            total: 5,
            feedback: "Good job".to_string(),
        }
    }

    fn sample_task() -> ReportTask {
        ReportTask {
            task_number: 1,
            name: "Task 1".to_string(),
            score: sample_score(),
            subsections: vec![sample_subsection()],
        }
    }

    #[test]
    fn test_score_struct_fields() {
        let score = Score { earned: 5, total: 10 };
        assert_eq!(score.earned, 5);
        assert_eq!(score.total, 10);
    }

    #[test]
    fn test_report_task_and_subsection() {
        let subsection = sample_subsection();
        let task = ReportTask {
            task_number: 2,
            name: "Task 2".to_string(),
            score: Score { earned: 7, total: 10 },
            subsections: vec![subsection.clone()],
        };
        assert_eq!(task.task_number, 2);
        assert_eq!(task.name, "Task 2");
        assert_eq!(task.score.earned, 7);
        assert_eq!(task.subsections[0].label, "Subtask 1");
    }

    #[test]
    fn test_mark_report_serialization() {
        let report = MarkReport {
            created_at: "2024-06-01T12:00:00Z".to_string(),
            updated_at: "2024-06-01T12:00:00Z".to_string(),
            mark: sample_score(),
            tasks: vec![sample_task()],
            code_coverage: None,
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"created_at\":\"2024-06-01T12:00:00Z\""));
        assert!(json.contains("\"updated_at\":\"2024-06-01T12:00:00Z\""));
        assert!(json.contains("Task 1"));
        assert!(json.contains("Subtask 1"));
    }

    #[test]
    fn test_mark_report_response_from_trait() {
        let report = MarkReport {
            created_at: "2024-06-01T12:00:00Z".to_string(),
            updated_at: "2024-06-01T12:00:00Z".to_string(),
            mark: sample_score(),
            tasks: vec![],
            code_coverage: None,
        };
        let response: MarkReportResponse = report.into();
        assert!(response.success);
        assert_eq!(response.message, "Grading complete.");
    }

    #[test]
    fn test_generate_new_mark_report_function() {
        let now = "2024-06-01T12:00:00Z".to_string();
        let mark = Score { earned: 10, total: 20 };
        let tasks = vec![sample_task()];
        let report = generate_new_mark_report(now.clone(), now.clone(), tasks.clone(), mark.clone());
        assert_eq!(report.created_at, now);
        assert_eq!(report.updated_at, now);
        assert_eq!(report.mark.earned, 10);
        assert_eq!(report.tasks.len(), 1);
    }

    #[test]
    fn test_code_coverage_optional_fields() {
        let coverage = CodeCoverageReport {
            summary: Some(Score { earned: 5, total: 10 }),
            files: vec![CoverageFile { path: "src/lib.rs".to_string(), earned: 5, total: 10 }],
        };
        let report = MarkReport {
            created_at: "2024-06-01T12:00:00Z".to_string(),
            updated_at: "2024-06-01T12:00:00Z".to_string(),
            mark: sample_score(),
            tasks: vec![],
            code_coverage: Some(coverage),
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("code_coverage"));
    }
}