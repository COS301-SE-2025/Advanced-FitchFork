//! # Types Module
//!
//! This module defines the core data structures used throughout the marker system.
//! These types are used to represent the results of marking tasks and other relevant data.

/// Represents the result of a single marking task.
///
/// This struct holds the information about a task's outcome, including the score awarded,
/// the maximum possible score, and details about what was matched or missed during comparison.
pub struct TaskResult {
    /// A descriptive name for the task.
    pub name: String,
    /// The number of points awarded for the task.
    pub awarded: u32,
    /// The maximum number of points possible for the task.
    pub possible: u32,
    /// The weight of this task's score in the overall calculation (should sum to 1.0 across all tasks).
    pub weight: f64,
    /// A list of patterns or items that were successfully matched in the student's output.
    pub matched_patterns: Vec<String>,
    /// A list of patterns or items that were expected but not found in the student's output.
    pub missed_patterns: Vec<String>,
} 