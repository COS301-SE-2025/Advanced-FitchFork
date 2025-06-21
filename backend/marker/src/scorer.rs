//! # Scorer Module
//!
//! This module provides functions for calculating scores based on the outcomes of various tasks.
//! The primary function, `compute_overall_score`, aggregates individual task results into a
//! single, final score.

use crate::error::MarkerError;
use crate::types::TaskResult;

const WEIGHT_TOLERANCE: f64 = 1e-9;

/// Computes the overall score from a slice of `TaskResult`s, taking into account task weights.
///
/// This function calculates a weighted average of the percentage scores of all tasks.
/// If the weights sum to 1.0, they are used as is. If they sum to 0.0, the tasks are weighted
/// equally. Otherwise, an error is returned. The final score is an integer between 0 and 100.
///
/// # Arguments
///
/// * `results` - A slice of `TaskResult` structs, each representing the outcome of a single task.
///
/// # Returns
///
/// A `Result` containing the final score as a `u32`.
/// - `Ok(u32)`: The final score, rounded to the nearest integer. If `results` is empty, returns `Ok(0)`.
/// - `Err(MarkerError::WeightMismatch)`: If the sum of weights is not close to 1.0 or 0.0.
///
/// # Behavior
///
/// - If the sum of `weight`s is close to 1.0, a weighted average is computed.
/// - If the sum of `weight`s is close to 0.0, tasks are weighted equally.
/// - Tasks where `possible` is 0 contribute 0% to the score.
/// - Returns a `WeightMismatch` error for invalid weight sums.
///
/// # Example
///
/// ```
/// use marker::types::TaskResult;
/// use marker::scorer::compute_overall_score;
///
/// let results = vec![
///     TaskResult { name: "Task 1".to_string(), awarded: 10, possible: 10, weight: 0.8, matched_patterns: vec![], missed_patterns: vec![] }, // 100%
///     TaskResult { name: "Task 2".to_string(), awarded: 5, possible: 10, weight: 0.2, matched_patterns: vec![], missed_patterns: vec![] },  // 50%
/// ];
///
/// // Weighted score: (1.0 * 0.8) + (0.5 * 0.2) = 0.8 + 0.1 = 0.9, which is 90%
/// let score = compute_overall_score(&results).unwrap();
/// assert_eq!(score, 90);
///
/// // Example with equal weights (summing to 0).
/// let results_equal_weight = vec![
///     TaskResult { name: "Task A".to_string(), awarded: 10, possible: 10, weight: 0.0, matched_patterns: vec![], missed_patterns: vec![] },
///     TaskResult { name: "Task B".to_string(), awarded: 5, possible: 10, weight: 0.0, matched_patterns: vec![], missed_patterns: vec![] },
/// ];
/// // Average score: (1.0 + 0.5) / 2 = 0.75, which is 75%
/// let score_equal = compute_overall_score(&results_equal_weight).unwrap();
/// assert_eq!(score_equal, 75);
/// ```
pub fn compute_overall_score(results: &[TaskResult]) -> Result<u32, MarkerError> {
    if results.is_empty() {
        return Ok(0);
    }

    let total_weight: f64 = results.iter().map(|r| r.weight).sum();

    let mut overall_score = 0.0;

    if (total_weight - 1.0).abs() < WEIGHT_TOLERANCE {
        for result in results {
            let percentage = if result.possible > 0 {
                result.awarded as f64 / result.possible as f64
            } else {
                0.0
            };
            overall_score += percentage * result.weight;
        }
    } else if (total_weight - 0.0).abs() < WEIGHT_TOLERANCE {
        let num_tasks = results.len() as f64;
        let equal_weight = 1.0 / num_tasks;
        for result in results {
            let percentage = if result.possible > 0 {
                result.awarded as f64 / result.possible as f64
            } else {
                0.0
            };
            overall_score += percentage * equal_weight;
        }
    } else {
        return Err(MarkerError::WeightMismatch(format!(
            "Sum of weights must be 1.0, but is {}",
            total_weight
        )));
    }

    Ok((overall_score * 100.0).round() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TaskResult;

    /// Test case 1: Valid weights that sum to 1.0.
    /// The score should be the weighted average of the task percentages.
    #[test]
    fn test_valid_weights_sum_to_one() {
        let results = vec![
            TaskResult { name: "Task 1".to_string(), awarded: 10, possible: 10, weight: 0.7, matched_patterns: vec![], missed_patterns: vec![] }, // 100% -> 70
            TaskResult { name: "Task 2".to_string(), awarded: 5,  possible: 10, weight: 0.3, matched_patterns: vec![], missed_patterns: vec![] }, // 50%  -> 15
        ];
        // Total score = (1.0 * 0.7) + (0.5 * 0.3) = 0.7 + 0.15 = 0.85 -> 85
        assert_eq!(compute_overall_score(&results).unwrap(), 85);
    }

    /// Test case 2: All weights are 0.0, triggering equal distribution.
    /// The score should be the simple average of the task percentages.
    #[test]
    fn test_zero_weights_equal_distribution() {
        let results = vec![
            TaskResult { name: "Task A".to_string(), awarded: 10, possible: 20, weight: 0.0, matched_patterns: vec![], missed_patterns: vec![] }, // 50%
            TaskResult { name: "Task B".to_string(), awarded: 20, possible: 20, weight: 0.0, matched_patterns: vec![], missed_patterns: vec![] }, // 100%
            TaskResult { name: "Task C".to_string(), awarded: 0,  possible: 20, weight: 0.0, matched_patterns: vec![], missed_patterns: vec![] }, // 0%
        ];
        // Total percentage = 0.5 + 1.0 + 0.0 = 1.5
        // Average = 1.5 / 3 = 0.5 -> 50
        assert_eq!(compute_overall_score(&results).unwrap(), 50);
    }

    /// Test case 3: The input list of results is empty.
    /// The function should return 0 without panicking.
    #[test]
    fn test_empty_results_list() {
        let results: Vec<TaskResult> = vec![];
        assert_eq!(compute_overall_score(&results).unwrap(), 0);
    }

    /// Test case 4: The weights do not sum to 1.0 or 0.0.
    /// This is an invalid state, and the function should return a `WeightMismatch` error.
    #[test]
    fn test_invalid_weight_sum() {
        let results = vec![
            TaskResult { name: "Invalid Task".to_string(), awarded: 10, possible: 10, weight: 0.5, matched_patterns: vec![], missed_patterns: vec![] },
        ];
        let result = compute_overall_score(&results);
        assert!(matches!(result, Err(MarkerError::WeightMismatch(_))));
    }

    /// Test case 5: One of the tasks has a `possible` score of 0.
    /// This task's percentage should be treated as 0 to avoid division by zero.
    #[test]
    fn test_task_with_zero_possible_score() {
        let results = vec![
            TaskResult { name: "Valid Task".to_string(),   awarded: 10, possible: 10, weight: 0.5, matched_patterns: vec![], missed_patterns: vec![] }, // 100% -> 50
            TaskResult { name: "Zero Poss Task".to_string(), awarded: 0,  possible: 0,  weight: 0.5, matched_patterns: vec![], missed_patterns: vec![] }, // 0%   -> 0
        ];
        // Total score = (1.0 * 0.5) + (0.0 * 0.5) = 0.5 -> 50
        assert_eq!(compute_overall_score(&results).unwrap(), 50);
    }

    /// Test case 6: The final score requires rounding.
    /// The result should be rounded to the nearest integer.
    #[test]
    fn test_rounding_logic() {
        let results = vec![
            TaskResult { name: "Task X".to_string(), awarded: 8, possible: 10, weight: 0.85, matched_patterns: vec![], missed_patterns: vec![] }, // 80%
            TaskResult { name: "Task Y".to_string(), awarded: 9, possible: 10, weight: 0.15, matched_patterns: vec![], missed_patterns: vec![] }, // 90%
        ];
        // Total score = (0.8 * 0.85) + (0.9 * 0.15) = 0.68 + 0.135 = 0.815 -> 82
        assert_eq!(compute_overall_score(&results).unwrap(), 82);

        let results_round_up = vec![
            TaskResult { name: "A".to_string(), awarded: 2, possible: 3, weight: 1.0, matched_patterns: vec![], missed_patterns: vec![] }, // 66.66...%
        ];
        // (2/3) * 100 = 66.66... -> rounded to 67
        assert_eq!(compute_overall_score(&results_round_up).unwrap(), 67);
    }
} 