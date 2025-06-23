use crate::types::{TaskResult, Subsection};

/// OutputComparator is a strategy trait for comparing outputs.
/// Each implementation provides a specific logic for comparing a single subsection
/// of a student's output against the reference output.
pub trait OutputComparator {
    /// Compare one subsection (pattern) of a task, producing a full TaskResult.
    ///
    /// - `section`: contains `name`, `value`.
    /// - `memo_lines`: the string/regex for this subsection.
    /// - `student_lines`: text to search.
    ///
    /// Returns the result as a `TaskResult`
    fn compare(
        &self,
        section: &Subsection,
        memo_lines: &[String],
        student_lines: &[String]
    ) -> TaskResult;
} 