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
use std::collections::HashMap;

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
                            summary
                                .push_str(&format!("Code crashed with exit code {}", return_code));
                        }
                    } else {
                        summary.push_str(&format!("Code crashed with exit code {}", return_code));
                    }
                }
            }

            if summary.is_empty() {
                fn freq_map(lines: &[String]) -> HashMap<&String, usize> {
                    let mut m = HashMap::new();
                    for s in lines {
                        *m.entry(s).or_insert(0) += 1;
                    }
                    m
                }

                let student_counts = freq_map(&result.student_output);
                let memo_counts = freq_map(&result.memo_output);

                let student_total = result.student_output.len();
                let memo_total = result.memo_output.len();

                let memo_is_multiset_subset = memo_counts
                    .iter()
                    .all(|(k, &v)| student_counts.get(k).unwrap_or(&0) >= &v);

                let student_is_multiset_subset = student_counts
                    .iter()
                    .all(|(k, &v)| memo_counts.get(k).unwrap_or(&0) >= &v);

                let multisets_equal = student_counts == memo_counts;

                let feedback_message = if memo_is_multiset_subset && memo_total < student_total {
                    "Too much output"
                } else if !result.missed_patterns.is_empty() {
                    if student_is_multiset_subset && student_total < memo_total {
                        "Missing lines"
                    } else {
                        "Incorrect output"
                    }
                } else if !result.matched_patterns.is_empty() {
                    if multisets_equal {
                        "All patterns matched"
                    } else {
                        "Incorrect output"
                    }
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(
        name: &str,
        matched: &[&str],
        missed: &[&str],
        awarded: f64,
        possible: f64,
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
            2.0,
            2.0,
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
            1.0,
            3.0,
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
            1.0,
            2.0,
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
            1.0,
            3.0,
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
            0.0,
            2.0,
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
        let task = make_task("Task4", &[], &[], 0.0, 0.0, &vec![], &vec![], None, None);
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
        let t1 = make_task("T1", &["a"], &[], 1.0, 1.0, &vec![], &vec![], None, None);
        let t2 = make_task("T2", &[], &["b"], 0.0, 1.0, &[], &["b"], None, None);
        let t3 = make_task(
            "T3",
            &["x"],
            &["y"],
            1.0,
            2.0,
            &["x"],
            &["x", "y"],
            None,
            None,
        );
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
            0.0,
            10.0,
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
            0.0,
            10.0,
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
