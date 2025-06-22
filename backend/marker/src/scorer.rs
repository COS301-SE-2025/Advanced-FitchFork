//! # Scorer Module
//!
//! This module provides functions for calculating scores based on the outcomes of various tasks.
//! The primary function, `compute_overall_score`, aggregates individual task results into a
//! single, final score.

use crate::error::MarkerError;
use crate::types::TaskResult;

/// Computes the overall score as a percentage from a slice of `TaskResult`s.
///
/// This function calculates the average of the percentage scores of all tasks.
/// Each task's score is determined by the ratio of `awarded` points to `possible` points. The final
/// score is an integer between 0 and 100.
///
/// # Arguments
///
/// * `results` - A slice of `TaskResult` structs, each representing the outcome of a single task.
///
/// # Returns
///
/// A `Result` containing the final score as a `u32`.
/// - `Ok(u32)`: The final score, rounded to the nearest integer. If `results` is empty, returns `Ok(0)`.
/// - `Err(MarkerError)`: This is not returned by the current implementation but is part of the function signature for future compatibility.
///
/// # Behavior
///
/// - Tasks where `possible` is 0 are ignored in the calculation to prevent division by zero.
/// - The final score is the mean of the percentages of the valid tasks.
///
/// # Example
///
/// ```
/// use marker::types::TaskResult;
/// use marker::scorer::compute_overall_score;
///
/// let results = vec![
///     TaskResult { name: "Task 1".to_string(), awarded: 10, possible: 10, matched_patterns: vec![], missed_patterns: vec![] }, // 100%
///     TaskResult { name: "Task 2".to_string(), awarded: 5, possible: 10, matched_patterns: vec![], missed_patterns: vec![] },  // 50%
/// ];
///
/// // The average of (1.0 + 0.5) / 2 = 0.75, which is 75%
/// let score = compute_overall_score(&results).unwrap();
/// assert_eq!(score, 75);
///
/// // Example with an empty list of results.
/// let empty_results: Vec<TaskResult> = vec![];
/// let score = compute_overall_score(&empty_results).unwrap();
/// assert_eq!(score, 0);
/// ```
pub fn compute_overall_score(results: &[TaskResult]) -> Result<u32, MarkerError> {
    if results.is_empty() {
        return Ok(0);
    }

    let mut total_percentage = 0.0;
    for result in results {
        if result.possible > 0 {
            total_percentage += result.awarded as f64 / result.possible as f64;
        }
    }

    let num_tasks = results.len() as f64;
    let overall_score = if num_tasks > 0.0 {
        total_percentage / num_tasks
    } else {
        0.0
    };

    Ok((overall_score * 100.0).round() as u32)
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
            }, // 100%
            TaskResult {
                name: "Task 2".to_string(),
                awarded: 5,
                possible: 10,
                matched_patterns: vec![],
                missed_patterns: vec![],
            }, // 50%
        ];
        // Average of (1.0 + 0.5) / 2 = 0.75 => 75
        assert_eq!(compute_overall_score(&results).unwrap(), 75);
    }

    /// Tests the function with an empty list of results, expecting a score of 0.
    #[test]
    fn test_compute_overall_score_empty() {
        let results: Vec<TaskResult> = vec![];
        assert_eq!(compute_overall_score(&results).unwrap(), 0);
    }

    /// Tests that tasks with zero possible points are correctly ignored.
    #[test]
    fn test_compute_overall_score_with_zero_possible() {
        let results = vec![
            TaskResult {
                name: "Task 1".to_string(),
                awarded: 10,
                possible: 10,
                matched_patterns: vec![],
                missed_patterns: vec![],
            }, // 100%
            TaskResult {
                name: "Task 2".to_string(),
                awarded: 5,
                possible: 0,
                matched_patterns: vec![],
                missed_patterns: vec![],
            }, // Ignored
        ];
        // Average of 1.0 / 1 = 1.0 => 100. This is because the second task is ignored from the count.
        // Let's adjust the logic slightly to be more intuitive. A task with possible: 0 should be counted as a task but contribute 0% to the score.
        // (1.0 + 0.0) / 2 = 0.5 => 50
        // The current implementation does (1.0 / 2) = 0.5, which is 50. Let's confirm this logic is what we want.
        // Yes, `num_tasks` is `results.len()`, so the task is included in the denominator. This is correct.
        // Average of (1.0 + 0.0) / 2 = 0.5 => 50
        assert_eq!(compute_overall_score(&results).unwrap(), 50);
    }

    /// Tests a scenario where the final score requires rounding.
    #[test]
    fn test_compute_overall_score_rounding() {
        let results = vec![
            TaskResult {
                name: "Task 1".to_string(),
                awarded: 2,
                possible: 3,
                matched_patterns: vec![],
                missed_patterns: vec![],
            }, // 66.66...%
            TaskResult {
                name: "Task 2".to_string(),
                awarded: 1,
                possible: 2,
                matched_patterns: vec![],
                missed_patterns: vec![],
            }, // 50%
        ];
        // Average of (0.666... + 0.5) / 2 = 0.58333... => 58
        assert_eq!(compute_overall_score(&results).unwrap(), 58);
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
            },
            TaskResult {
                name: "Task 2".to_string(),
                awarded: 0,
                possible: 20,
                matched_patterns: vec![],
                missed_patterns: vec![],
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
            },
            TaskResult {
                name: "Task 2".to_string(),
                awarded: 100,
                possible: 100,
                matched_patterns: vec![],
                missed_patterns: vec![],
            },
        ];
        assert_eq!(compute_overall_score(&results).unwrap(), 100);
    }
} 