use crate::types::TaskResult;
use util::mark_allocator;

/// OutputComparator is a strategy trait for comparing outputs.
/// Each implementation provides a specific logic for comparing a single subsection
/// of a student's output against the reference output.
pub trait OutputComparator: Send + Sync {
    /// Compare one subsection (pattern) of a task, producing a full TaskResult.
    ///
    /// - `section`: util allocator subsection (name, value, regex?).
    /// - `memo_lines`: the string/regex for this subsection.
    /// - `student_lines`: text to compare.
    fn compare(
        &self,
        section: &mark_allocator::Subsection,
        memo_lines: &[String],
        student_lines: &[String],
    ) -> TaskResult;
}
