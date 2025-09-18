//! # Scorer Module
//!
//! This module provides functions for calculating scores based on the outcomes of various tasks.
//! The primary function, `compute_overall_score`, aggregates individual task results into a
//! single, final score.

use crate::error::MarkerError;
use crate::types::TaskResult;

/// Computes the overall score as the sum of awarded points from a slice of `TaskResult`s.
///
/// This function calculates the total awarded points of all tasks.
/// The final score is an integer representing the sum of awarded points.
///
/// # Arguments
///
/// * `results` - A slice of `TaskResult` structs, each representing the outcome of a single task.
///
/// # Returns
///
/// A `Result` containing the final score as a `u32`.
/// - `Ok(u32)`: The final score, which is the sum of awarded points. If `results` is empty, returns `Ok(0)`.
/// - `Err(MarkerError)`: This is not returned by the current implementation but is part of the function signature for future compatibility.
///
/// # Example
///
/// ```
/// use marker::types::TaskResult;
/// use marker::scorer::compute_overall_score;
///
/// let results = vec![
///     TaskResult { name: "Task 1".to_string(), awarded: 10, possible: 10, matched_patterns: vec![], missed_patterns: vec![], student_output: vec![], memo_output: vec![] },
///     TaskResult { name: "Task 2".to_string(), awarded: 5, possible: 10, matched_patterns: vec![], missed_patterns: vec![], student_output: vec![], memo_output: vec![] },
/// ];
///
/// // Total awarded: 15
/// let score = compute_overall_score(&results).unwrap();
/// assert_eq!(score, 15);
///
/// // Example with an empty list of results.
/// let empty_results: Vec<TaskResult> = vec![];
/// let score = compute_overall_score(&empty_results).unwrap();
/// assert_eq!(score, 0);
/// ```
pub fn compute_overall_score(results: &[TaskResult]) -> Result<i64, MarkerError> {
    let mut total_awarded = 0;
    for result in results {
        total_awarded += result.awarded;
    }
    Ok(total_awarded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TaskResult;

    /// Tests the basic functionality with a standard set of tasks.
    #[test]
    fn test_compute_overall_score_basic() {
        let results = vec![
            TaskResult {
                name: "Task 1".to_string(),
                awarded: 10,
                possible: 10,
                matched_patterns: vec![],
                missed_patterns: vec![],
                student_output: vec![],
                memo_output: vec![],
                stderr: None,
                return_code: None,
                manual_feedback: None,
            },
            TaskResult {
                name: "Task 2".to_string(),
                awarded: 5,
                possible: 10,
                matched_patterns: vec![],
                missed_patterns: vec![],
                student_output: vec![],
                memo_output: vec![],
                stderr: None,
                return_code: None,
                manual_feedback: None,
            },
        ];
        assert_eq!(compute_overall_score(&results).unwrap(), 15);
    }

    /// Tests the function with an empty list of results, expecting a score of 0.
    #[test]
    fn test_compute_overall_score_empty() {
        let results: Vec<TaskResult> = vec![];
        assert_eq!(compute_overall_score(&results).unwrap(), 0);
    }

    /// Tests that tasks with zero possible points are still included if awarded > 0.
    #[test]
    fn test_compute_overall_score_with_zero_possible() {
        let results = vec![
            TaskResult {
                name: "Task 1".to_string(),
                awarded: 10,
                possible: 10,
                matched_patterns: vec![],
                missed_patterns: vec![],
                student_output: vec![],
                memo_output: vec![],
                stderr: None,
                return_code: None,
                manual_feedback: None,
            },
            TaskResult {
                name: "Task 2".to_string(),
                awarded: 5,
                possible: 0,
                matched_patterns: vec![],
                missed_patterns: vec![],
                student_output: vec![],
                memo_output: vec![],
                stderr: None,
                return_code: None,
                manual_feedback: None,
            },
        ];
        assert_eq!(compute_overall_score(&results).unwrap(), 15);
    }

    #[test]
    fn test_compute_overall_score_sum() {
        let results = vec![
            TaskResult {
                name: "Task 1".to_string(),
                awarded: 2,
                possible: 3,
                matched_patterns: vec![],
                missed_patterns: vec![],
                student_output: vec![],
                memo_output: vec![],
                stderr: None,
                return_code: None,
                manual_feedback: None,
            },
            TaskResult {
                name: "Task 2".to_string(),
                awarded: 1,
                possible: 2,
                matched_patterns: vec![],
                missed_patterns: vec![],
                student_output: vec![],
                memo_output: vec![],
                stderr: None,
                return_code: None,
                manual_feedback: None,
            },
        ];
        assert_eq!(compute_overall_score(&results).unwrap(), 3);
    }

    /// Tests the case where all tasks score zero.
    #[test]
    fn test_compute_overall_score_all_zero() {
        let results = vec![
            TaskResult {
                name: "Task 1".to_string(),
                awarded: 0,
                possible: 10,
                matched_patterns: vec![],
                missed_patterns: vec![],
                student_output: vec![],
                memo_output: vec![],
                stderr: None,
                return_code: None,
                manual_feedback: None,
            },
            TaskResult {
                name: "Task 2".to_string(),
                awarded: 0,
                possible: 20,
                matched_patterns: vec![],
                missed_patterns: vec![],
                student_output: vec![],
                memo_output: vec![],
                stderr: None,
                return_code: None,
                manual_feedback: None,
            },
        ];
        assert_eq!(compute_overall_score(&results).unwrap(), 0);
    }

    /// Tests the case where all tasks receive a perfect score.
    #[test]
    fn test_compute_overall_score_all_perfect() {
        let results = vec![
            TaskResult {
                name: "Task 1".to_string(),
                awarded: 15,
                possible: 15,
                matched_patterns: vec![],
                missed_patterns: vec![],
                student_output: vec![],
                memo_output: vec![],
                stderr: None,
                return_code: None,
                manual_feedback: None,
            },
            TaskResult {
                name: "Task 2".to_string(),
                awarded: 100,
                possible: 100,
                matched_patterns: vec![],
                missed_patterns: vec![],
                student_output: vec![],
                memo_output: vec![],
                stderr: None,
                return_code: None,
                manual_feedback: None,
            },
        ];
        assert_eq!(compute_overall_score(&results).unwrap(), 115);
    }
} 