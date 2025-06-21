/// OutputComparator is a strategy trait for comparing outputs.
/// Each implementation provides a specific logic for comparing a single subsection
/// of a student's output against the reference output.
pub trait OutputComparator {
    /// Compares a single subsection of student output against the memo.
    ///
    /// - `memo_lines`: All lines from the reference (memo) output.
    /// - `student_lines`: All lines from the student's output.
    /// - `pattern`: The specific string pattern for the subsection to find.
    /// - `max_marks`: The maximum marks available for this subsection.
    ///
    /// Returns the awarded marks as a u32.
    fn compare(&self, memo_lines: &[String], student_lines: &[String], pattern: &str, max_marks: u32) -> u32;
} 