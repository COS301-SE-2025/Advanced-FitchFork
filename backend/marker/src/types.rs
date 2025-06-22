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
    /// A list of patterns or items that were successfully matched in the student's output.
    pub matched_patterns: Vec<String>,
    /// A list of patterns or items that were expected but not found in the student's output.
    pub missed_patterns: Vec<String>,
}

/// The top-level schema for an allocator report, containing a list of tasks.
#[derive(Debug)]
pub struct AllocatorSchema(pub Vec<TaskEntry>);

/// Represents a single task in the allocator report.
#[derive(Debug)]
pub struct TaskEntry {
    /// The task identifier (e.g., "task1").
    pub id: String,
    /// The name of the task.
    pub name: String,
    /// The value (score/points) assigned to the task.
    pub value: u32,
    /// The subsections of the task. Every task must have atleast one subsection.
    pub subsections: Vec<Subsection>,
}

/// Represents a subsection within a task.
#[derive(Debug)]
pub struct Subsection {
    /// The name of the subsection.
    pub name: String,
    /// The value (score/points) assigned to the subsection.
    pub value: u32,
} 