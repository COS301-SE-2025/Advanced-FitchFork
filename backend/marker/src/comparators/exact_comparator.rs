//! A comparator that performs an exact match comparison between memo and student output.
//!
//! The `ExactComparator` is designed to award marks on an all-or-nothing basis. It checks if a
//! specific pattern appears in the student's output at least as many times as it appears in the
//! memo (solution) output.

use crate::traits::comparator::OutputComparator;

/// A comparator that awards full marks if the student's output contains a pattern at least as
/// many times as the memo's output.
///
/// This comparator is useful for tasks where the presence and frequency of a specific output line
/// or pattern is a critical success factor. If the expected pattern appears one or more times in
/// the memo, the student's output must contain it at least that many times to receive any marks.
pub struct ExactComparator;

impl OutputComparator for ExactComparator {
    /// Compares the student's output with the memo's output based on the occurrence count of a pattern.
    ///
    /// # Arguments
    ///
    /// * `memo_lines` - A slice of strings representing the lines of the memo output.
    /// * `student_lines` - A slice of strings representing the lines of the student's output.
    /// * `pattern` - The exact string pattern to search for in the output lines.
    /// * `max_marks` - The total marks to award if the comparison is successful.
    ///
    /// # Returns
    ///
    /// Returns `max_marks` if the `pattern` is found in `memo_lines` at least once, and the count
    /// of `pattern` in `student_lines` is greater than or equal to its count in `memo_lines`.
    /// Otherwise, it returns `0`.
    fn compare(&self, memo_lines: &[String], student_lines: &[String], pattern: &str, max_marks: u32) -> u32 {
        let memo_count = memo_lines.iter().filter(|l| l.contains(pattern)).count();
        let student_count = student_lines.iter().filter(|l| l.contains(pattern)).count();

        if memo_count > 0 && student_count >= memo_count {
            max_marks
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a vector of strings from a slice of string literals.
    fn to_string_vec(lines: &[&str]) -> Vec<String> {
        lines.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_perfect_match() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["apple", "orange", "apple"]);
        let student_lines = to_string_vec(&["apple", "apple", "grape"]);
        let pattern = "apple";
        let max_marks = 10;
        // memo_count = 2, student_count = 2. Should be a match.
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 10);
    }

    #[test]
    fn test_student_has_more_matches() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["one match"]);
        let student_lines = to_string_vec(&["one match", "two matches", "three matches"]);
        let pattern = "match";
        let max_marks = 5;
        // memo_count = 1, student_count = 3. Student has more, so should match.
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 5);
    }

    #[test]
    fn test_student_has_fewer_matches() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["correct", "correct", "correct"]);
        let student_lines = to_string_vec(&["correct", "wrong"]);
        let pattern = "correct";
        let max_marks = 20;
        // memo_count = 3, student_count = 1. Student has fewer, so should not match.
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 0);
    }

    #[test]
    fn test_no_matches_in_memo() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["no expected output", "none"]);
        let student_lines = to_string_vec(&["unexpected", "unexpected"]);
        let pattern = "unexpected";
        let max_marks = 10;
        // memo_count = 0. Should not match, regardless of student output.
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 0);
    }

    #[test]
    fn test_no_matches_in_student() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["required", "required"]);
        let student_lines = to_string_vec(&["something else"]);
        let pattern = "required";
        let max_marks = 15;
        // memo_count = 2, student_count = 0. Should not match.
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 0);
    }

    #[test]
    fn test_no_matches_in_either() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["empty"]);
        let student_lines = to_string_vec(&["empty"]);
        let pattern = "unique";
        let max_marks = 10;
        // memo_count = 0, student_count = 0. Should not match.
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 0);
    }
} 