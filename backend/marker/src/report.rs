//! # Mark Report Module
//!
//! This module defines the data structures and response envelope for returning grading results from the marker system.
//! It provides a standardized, serializable format for reporting per-task results, overall score, and feedback to clients.
//!
//! ## Overview
//!
//! The main types are:
//! - [`MarkReport`]: Contains all grading data for a submission, including per-task results, feedback, and overall score.
//! - [`MarkReportResponse`]: A response envelope that wraps a [`MarkReport`] with success and message fields for API responses.
//!
//! ## JSON Output Example
//!
//! When serialized, the response will look like:
//!
//! ```json
//! {
//!   "success": true,
//!   "message": "Grading complete.",
//!   "data": {
//!     "submission_id": "...",
//!     "task_results": [
//!       { "name": "...", "awarded": 5, "possible": 10, "percentage": 50.0 },
//!       ...
//!     ],
//!     "overall_score": 75,
//!     "feedback": [
//!       { "task": "...", "message": "..." },
//!       ...
//!     ]
//!   }
//! }
//! ```
//!
//! ## Design Notes
//!
//! - [`MarkReport`] is intended for API output. It contains only serializable fields and is not used for internal grading logic.
//! - The [`From<MarkReport> for MarkReportResponse`] implementation provides ergonomic conversion for API handlers.

use crate::{traits::feedback::FeedbackEntry, types::JsonTaskResult};
use serde::Serialize;

/// Represents the final report generated after marking a submission.
///
/// This struct is designed for API output and contains all information needed to present grading results
/// to the client, including per-task results, overall score, and feedback.
///
/// - `submission_id`: The unique identifier for the submission.
/// - `overall_score`: The overall score as a percentage (0-100).
/// - `feedback`: A list of feedback entries for the submission.
/// - `task_results`: A list of per-task results, each including name, awarded, possible, and percentage.
#[derive(Debug, Serialize)]
pub struct MarkReport {
    /// The unique identifier for the submission.
    pub submission_id: String,
    /// The overall score as a percentage (0-100).
    pub overall_score: u32,
    /// A list of feedback entries for the submission.
    pub feedback: Vec<FeedbackEntry>,
    /// A list of task results for the submission, each with computed percentage.
    pub task_results: Vec<JsonTaskResult>,
}

/// The API response envelope for grading results.
///
/// This struct wraps a [`MarkReport`] and adds top-level `success` and `message` fields for consistency
/// with other API responses.
///
/// - `success`: Always true for successful grading.
/// - `message`: A human-readable message (e.g., "Grading complete.").
/// - `data`: The [`MarkReport`] containing all grading details.
#[derive(Debug, Serialize)]
pub struct MarkReportResponse {
    /// Indicates the grading was successful.
    success: bool,
    /// A human-readable message for the client.
    message: String,
    /// The detailed grading report.
    data: MarkReport,
}

/// Enables ergonomic conversion from [`MarkReport`] to [`MarkReportResponse`].
impl From<MarkReport> for MarkReportResponse {
    fn from(report: MarkReport) -> Self {
        MarkReportResponse {
            success: true,
            message: "Grading complete.".to_string(),
            data: report,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_mark_report_response_serialization() {
        let report = MarkReport {
            submission_id: "abc123".to_string(),
            overall_score: 88,
            feedback: vec![FeedbackEntry { task: "Task1".to_string(), message: "Well done".to_string() }],
            task_results: vec![JsonTaskResult {
                name: "Task1".to_string(),
                awarded: 8,
                possible: 10,
                percentage: 80.0,
            }],
        };
        let response: MarkReportResponse = report.into();
        let value: Value = serde_json::to_value(&response).unwrap();
        assert_eq!(value["success"], true);
        assert_eq!(value["message"], "Grading complete.");
        assert_eq!(value["data"]["submission_id"], "abc123");
        assert_eq!(value["data"]["overall_score"], 88);
        assert_eq!(value["data"]["task_results"][0]["name"], "Task1");
        assert_eq!(value["data"]["task_results"][0]["awarded"], 8);
        assert_eq!(value["data"]["task_results"][0]["possible"], 10);
        assert_eq!(value["data"]["task_results"][0]["percentage"], 80.0);
        assert_eq!(value["data"]["feedback"][0]["task"], "Task1");
        assert_eq!(value["data"]["feedback"][0]["message"], "Well done");
    }

    #[test]
    fn test_percentage_computation_and_serialization() {
        let report = MarkReport {
            submission_id: "id".to_string(),
            overall_score: 100,
            feedback: vec![],
            task_results: vec![JsonTaskResult {
                name: "T2".to_string(),
                awarded: 5,
                possible: 10,
                percentage: 50.0,
            }, JsonTaskResult {
                name: "T3".to_string(),
                awarded: 0,
                possible: 0,
                percentage: 0.0,
            }],
        };
        let response: MarkReportResponse = report.into();
        let value: Value = serde_json::to_value(&response).unwrap();
        assert_eq!(value["data"]["task_results"][0]["percentage"], 50.0);
        assert_eq!(value["data"]["task_results"][1]["percentage"], 0.0);
    }

    #[test]
    fn test_feedback_serialization() {
        let report = MarkReport {
            submission_id: "id2".to_string(),
            overall_score: 77,
            feedback: vec![
                FeedbackEntry { task: "T1".to_string(), message: "msg1".to_string() },
                FeedbackEntry { task: "T2".to_string(), message: "msg2".to_string() },
            ],
            task_results: vec![],
        };
        let response: MarkReportResponse = report.into();
        let value: Value = serde_json::to_value(&response).unwrap();
        assert_eq!(value["data"]["feedback"].as_array().unwrap().len(), 2);
        assert_eq!(value["data"]["feedback"][1]["task"], "T2");
        assert_eq!(value["data"]["feedback"][1]["message"], "msg2");
    }

    #[test]
    fn test_empty_report_serialization() {
        let report = MarkReport {
            submission_id: "empty".to_string(),
            overall_score: 0,
            feedback: vec![],
            task_results: vec![],
        };
        let response: MarkReportResponse = report.into();
        let value: Value = serde_json::to_value(&response).unwrap();
        assert_eq!(value["data"]["submission_id"], "empty");
        assert_eq!(value["data"]["overall_score"], 0);
        assert!(value["data"]["feedback"].as_array().unwrap().is_empty());
        assert!(value["data"]["task_results"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_round_trip_json() {
        let report = MarkReport {
            submission_id: "roundtrip".to_string(),
            overall_score: 42,
            feedback: vec![FeedbackEntry { task: "A".to_string(), message: "B".to_string() }],
            task_results: vec![JsonTaskResult {
                name: "T".to_string(),
                awarded: 1,
                possible: 2,
                percentage: 50.0,
            }],
        };
        let response: MarkReportResponse = report.into();
        let json = serde_json::to_string(&response).unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["data"]["submission_id"], "roundtrip");
        assert_eq!(value["data"]["task_results"][0]["name"], "T");
        assert_eq!(value["data"]["feedback"][0]["task"], "A");
    }
} 