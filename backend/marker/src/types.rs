//! # Types Module
//!
//! This module defines the core data structures used throughout the marker system.
//! These types are used to represent the results of marking tasks and other relevant data.

use serde::Serialize;

// Re-export allocator schema from util so callers can `use crate::types::*;`
use util::mark_allocator as util_alloc;
pub type Allocator = util_alloc::MarkAllocator;
pub type Task = util_alloc::Task;
pub type Subsection = util_alloc::Subsection;

/// Represents the result of a single marking task.
///
/// This struct holds the information about a task's outcome, including the score awarded,
/// the maximum possible score, and details about what was matched or missed during comparison.
#[derive(Clone, Debug)]
pub struct TaskResult {
    /// A descriptive name for the task.
    pub name: String,
    /// The number of points awarded for the task.
    pub awarded: i64,
    /// The maximum number of points possible for the task.
    pub possible: i64,
    /// A list of patterns or items that were successfully matched in the student's output.
    pub matched_patterns: Vec<String>,
    /// A list of patterns or items that were expected but not found in the student's output.
    pub missed_patterns: Vec<String>,
    /// The student's actual output lines for comparison purposes.
    pub student_output: Vec<String>,
    /// The memo's expected output lines for comparison purposes.
    pub memo_output: Vec<String>,
    /// The stderr output (stack trace) if the student's code crashed.
    pub stderr: Option<String>,
    /// The return code from running the student's code.
    pub return_code: Option<i32>,
    /// Optional manual feedback message for manual feedback strategy.
    pub manual_feedback: Option<String>,
}

/// Represents a serializable per-task result for API output.
///
/// This struct is used in API responses to present the grading result for a single task, including the computed percentage score.
#[derive(Debug, Serialize)]
pub struct JsonTaskResult {
    /// The name of the task.
    pub name: String,
    /// The number of marks awarded for this task.
    pub awarded: i64,
    /// The maximum possible marks for this task.
    pub possible: i64,
    /// The percentage score for this task, computed as (awarded / possible) * 100 (or 0.0 if possible is zero).
    pub percentage: f32,
}