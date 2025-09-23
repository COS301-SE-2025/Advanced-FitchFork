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
use std::pin::Pin;

/// Automatic feedback strategy: generates template-based feedback for each task.
///
/// - Produces a summary of matched patterns and marks awarded for each task.
/// - Lists any missed patterns for each task.
/// - Implements the [`Feedback`] trait for use in the marker system.
#[derive(Debug)]
pub struct AutoFeedback;

impl Feedback for AutoFeedback {
    fn assemble_feedback<'a>(
        &'a self,
        results: &'a [TaskResult],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<FeedbackEntry>, MarkerError>> + Send + 'a>> {
        Box::pin(async move {
            let mut feedback_entries = Vec::new();

            for result in results {
                let mut summary = String::new();

                if let Some(return_code) = result.return_code {
                    if return_code != 0 {
                        if let Some(stderr) = &result.stderr {
                            if !stderr.trim().is_empty() {
                                summary.push_str(&format!(
                                    "Code crashed with exit code {}: {}",
                                    return_code,
                                    stderr.trim()
                                ));
                            } else {
                                summary.push_str(&format!(
                                    "Code crashed with exit code {}",
                                    return_code
                                ));
                            }
                        } else {
                            summary
                                .push_str(&format!("Code crashed with exit code {}", return_code));
                        }
                    }
                }

                if summary.is_empty() {
                    let student_set: std::collections::HashSet<&String> =
                        result.student_output.iter().collect();
                    let memo_set: std::collections::HashSet<&String> =
                        result.memo_output.iter().collect();

                    let feedback_message = if memo_set.is_subset(&student_set)
                        && memo_set.len() < student_set.len()
                    {
                        println!("TOO MUCH OUTPUT DETECTED");
                        "Too much output"
                    } else if !result.missed_patterns.is_empty() {
                        if student_set.is_subset(&memo_set) && student_set.len() < memo_set.len() {
                            "Missing lines"
                        } else {
                            "Incorrect output"
                        }
                    } else if !result.matched_patterns.is_empty() {
                        "All patterns matched"
                    } else {
                        ""
                    };

                    summary.push_str(feedback_message);
                }

                feedback_entries.push(FeedbackEntry {
                    task: result.name.clone(),
                    message: summary,
                });
            }

            Ok(feedback_entries)
        })
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
        student_output: &[&str],
        memo_output: &[&str],
        stderr: Option<&str>,
        return_code: Option<i32>,
    ) -> TaskResult {
        TaskResult {
            name: name.to_string(),
            awarded,
            possible,
            matched_patterns: matched.iter().map(|s| s.to_string()).collect(),
            missed_patterns: missed.iter().map(|s| s.to_string()).collect(),
            student_output: student_output.iter().map(|s| s.to_string()).collect(),
            memo_output: memo_output.iter().map(|s| s.to_string()).collect(),
            stderr: stderr.map(|s| s.to_string()),
            return_code,
            manual_feedback: None,
        }
    }

    #[tokio::test]
    async fn test_all_patterns_matched() {
        let task = make_task(
            "Task1",
            &["a", "b"],
            &[],
            2,
            2,
            &vec![],
            &vec![],
            None,
            None,
        );
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
    async fn test_some_patterns_missed_missing_lines() {
        let task = make_task(
            "Task2",
            &["a"],
            &["b", "c"],
            1,
            3,
            &["a"],
            &["a", "b", "c"],
            None,
            None,
        );
        let feedback = AutoFeedback.assemble_feedback(&[task]).await.unwrap();
        assert_eq!(
            feedback,
            vec![FeedbackEntry {
                task: "Task2".to_string(),
                message: "Missing lines".to_string(),
            },]
        );
    }

    #[tokio::test]
    async fn test_some_patterns_missed_too_much_output() {
        let task = make_task(
            "Task2",
            &["a"],
            &["b"],
            1,
            2,
            &["a", "b", "extra"],
            &["a", "b"],
            None,
            None,
        );
        let feedback = AutoFeedback.assemble_feedback(&[task]).await.unwrap();
        assert_eq!(
            feedback,
            vec![FeedbackEntry {
                task: "Task2".to_string(),
                message: "Too much output".to_string(),
            },]
        );
    }

    #[tokio::test]
    async fn test_some_patterns_missed_incorrect_output() {
        let task = make_task(
            "Task2",
            &["a"],
            &["b", "c"],
            1,
            3,
            &["a", "x"],
            &["a", "b", "c"],
            None,
            None,
        );
        let feedback = AutoFeedback.assemble_feedback(&[task]).await.unwrap();
        assert_eq!(
            feedback,
            vec![FeedbackEntry {
                task: "Task2".to_string(),
                message: "Incorrect output".to_string(),
            },]
        );
    }

    #[tokio::test]
    async fn test_only_missed_patterns() {
        let task = make_task(
            "Task3",
            &[],
            &["x", "y"],
            0,
            2,
            &[],
            &["x", "y"],
            None,
            None,
        );
        let feedback = AutoFeedback.assemble_feedback(&[task]).await.unwrap();
        assert_eq!(
            feedback,
            vec![FeedbackEntry {
                task: "Task3".to_string(),
                message: "Missing lines".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn test_empty_patterns() {
        let task = make_task("Task4", &[], &[], 0, 0, &vec![], &vec![], None, None);
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
        let t1 = make_task("T1", &["a"], &[], 1, 1, &vec![], &vec![], None, None);
        let t2 = make_task("T2", &[], &["b"], 0, 1, &[], &["b"], None, None);
        let t3 = make_task("T3", &["x"], &["y"], 1, 2, &["x"], &["x", "y"], None, None);
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
                    message: "Missing lines".to_string(),
                },
                FeedbackEntry {
                    task: "T3".to_string(),
                    message: "Missing lines".to_string(),
                },
            ]
        );
    }

    #[tokio::test]
    async fn test_crash_feedback_with_stderr() {
        let task = make_task(
            "CrashTask",
            &[],
            &["expected_output"],
            0,
            10,
            &[],
            &["expected_output"],
            Some(
                "java.lang.NullPointerException: Cannot invoke \"String.length()\" because \"str\" is null\n    at Main.main(Main.java:5)",
            ),
            Some(1),
        );
        let feedback = AutoFeedback.assemble_feedback(&[task]).await.unwrap();
        assert_eq!(feedback.len(), 1);
        assert!(
            feedback[0]
                .message
                .contains("Code crashed with exit code 1")
        );
        assert!(feedback[0].message.contains("NullPointerException"));
    }

    #[tokio::test]
    async fn test_crash_feedback_no_stderr() {
        let task = make_task(
            "CrashTask",
            &[],
            &["expected_output"],
            0,
            10,
            &[],
            &["expected_output"],
            None,
            Some(139),
        );
        let feedback = AutoFeedback.assemble_feedback(&[task]).await.unwrap();
        assert_eq!(feedback.len(), 1);
        assert_eq!(feedback[0].message, "Code crashed with exit code 139");
    }
}
