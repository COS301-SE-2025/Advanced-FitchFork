//! # AutoFeedback Strategy
//!
//! This module provides the `AutoFeedback` strategy for the marker system.
//! It implements the [`Feedback`] trait to generate automatic, template-based feedback for each task
//! based on matched and missed patterns in the marking results.
//!
//! ## Overview
//!
//! - For each task, if any patterns are matched, a feedback entry is generated summarizing the number of matched patterns and marks awarded.
//! - If any patterns are missed, a feedback entry is generated listing the missing patterns.
//! - Tasks with no matched or missed patterns produce no feedback.
//!
//! This strategy is useful for providing immediate, objective feedback to students based on their output.

use crate::error::MarkerError;
use crate::traits::feedback::{Feedback, FeedbackEntry};
use crate::types::TaskResult;
use async_trait::async_trait;

/// Automatic feedback strategy: generates template-based feedback for each task.
///
/// - Produces a summary of matched patterns and marks awarded for each task.
/// - Lists any missed patterns for each task.
/// - Implements the [`Feedback`] trait for use in the marker system.
#[derive(Debug)]
pub struct AutoFeedback;

#[async_trait]
impl Feedback for AutoFeedback {
    async fn assemble_feedback(
        &self,
        results: &[TaskResult],
    ) -> Result<Vec<FeedbackEntry>, MarkerError> {
        let mut feedback_entries = Vec::new();

        for result in results {
            let mut summary = String::new();
            if !result.missed_patterns.is_empty() {
                summary.push_str(&format!("Missing: {}", result.missed_patterns.join(", ")));
            } else if !result.matched_patterns.is_empty() {
                summary.push_str("All patterns matched");
            }
            feedback_entries.push(FeedbackEntry {
                task: result.name.clone(),
                message: summary,
            });
        }

        Ok(feedback_entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(
        name: &str,
        matched: &[&str],
        missed: &[&str],
        awarded: i64,
        possible: i64,
    ) -> TaskResult {
        TaskResult {
            name: name.to_string(),
            awarded,
            possible,
            matched_patterns: matched.iter().map(|s| s.to_string()).collect(),
            missed_patterns: missed.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[tokio::test]
    async fn test_all_patterns_matched() {
        let task = make_task("Task1", &["a", "b"], &[], 2, 2);
        let feedback = AutoFeedback.assemble_feedback(&[task]).await.unwrap();
        assert_eq!(
            feedback,
            vec![FeedbackEntry {
                task: "Task1".to_string(),
                message: "All patterns matched".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn test_some_patterns_missed() {
        let task = make_task("Task2", &["a"], &["b", "c"], 1, 3);
        let feedback = AutoFeedback.assemble_feedback(&[task]).await.unwrap();
        assert_eq!(
            feedback,
            vec![FeedbackEntry {
                task: "Task2".to_string(),
                message: "Missing: b, c".to_string(),
            },]
        );
    }

    #[tokio::test]
    async fn test_only_missed_patterns() {
        let task = make_task("Task3", &[], &["x", "y"], 0, 2);
        let feedback = AutoFeedback.assemble_feedback(&[task]).await.unwrap();
        assert_eq!(
            feedback,
            vec![FeedbackEntry {
                task: "Task3".to_string(),
                message: "Missing: x, y".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn test_empty_patterns() {
        let task = make_task("Task4", &[], &[], 0, 0);
        let feedback = AutoFeedback.assemble_feedback(&[task]).await.unwrap();
        assert_eq!(
            feedback,
            vec![FeedbackEntry {
                task: "Task4".to_string(),
                message: "".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn test_multiple_tasks() {
        let t1 = make_task("T1", &["a"], &[], 1, 1);
        let t2 = make_task("T2", &[], &["b"], 0, 1);
        let t3 = make_task("T3", &["x"], &["y"], 1, 2);
        let feedback = AutoFeedback.assemble_feedback(&[t1, t2, t3]).await.unwrap();
        assert_eq!(
            feedback,
            vec![
                FeedbackEntry {
                    task: "T1".to_string(),
                    message: "All patterns matched".to_string(),
                },
                FeedbackEntry {
                    task: "T2".to_string(),
                    message: "Missing: b".to_string(),
                },
                FeedbackEntry {
                    task: "T3".to_string(),
                    message: "Missing: y".to_string(),
                },
            ]
        );
    }
}
