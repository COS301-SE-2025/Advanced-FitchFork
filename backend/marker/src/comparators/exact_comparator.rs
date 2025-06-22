//! A comparator that performs an exact match comparison between memo and student output.
//!
//! The `ExactComparator` is designed to award marks on an all-or-nothing basis. It checks if a
//! specific pattern appears in the student's output at least as many times as it appears in the
//! memo (solution) output.

use crate::traits::comparator::OutputComparator;
use crate::types::{TaskResult, Subsection};

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
    /// * `section` - The subsection entry containing details like name and total possible value.
    /// * `memo_lines` - A slice of strings representing the lines of the memo output.
    /// * `student_lines` - A slice of strings representing the lines of the student's output.
    /// * `pattern` - The exact string pattern to search for in the output lines.
    ///
    /// # Returns
    ///
    /// Returns a `TaskResult` with full marks if the comparison is successful, or zero marks otherwise.
    fn compare(
        &self,
        section: &Subsection,
        memo_lines: &[String],
        student_lines: &[String],
        pattern: &str,
    ) -> TaskResult {
        let memo_count = memo_lines.iter().filter(|l| l.contains(pattern)).count();
        let student_count = student_lines.iter().filter(|l| l.contains(pattern)).count();

        let (awarded, matched_patterns, missed_patterns) =
            if memo_count > 0 && student_count >= memo_count {
                (section.value, vec![pattern.to_string()], vec![])
            } else {
                (0, vec![], vec![pattern.to_string()])
            };

        TaskResult {
            name: section.name.clone(),
            awarded,
            possible: section.value,
            matched_patterns,
            missed_patterns,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Subsection;

    /// Helper function to create a vector of strings from a slice of string literals.
    fn to_string_vec(lines: &[&str]) -> Vec<String> {
        lines.iter().map(|s| s.to_string()).collect()
    }

    fn mock_subsection(value: u32) -> Subsection {
        Subsection {
            name: "Mock Subsection".to_string(),
            value,
        }
    }

    #[test]
    fn test_perfect_match() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["apple", "orange", "apple"]);
        let student_lines = to_string_vec(&["apple", "apple", "grape"]);
        let pattern = "apple";
        let section = mock_subsection(10);
        // memo_count = 2, student_count = 2. Should be a match.
        let result = comparator.compare(&section, &memo_lines, &student_lines, pattern);
        assert_eq!(result.awarded, 10);
        assert_eq!(result.matched_patterns, vec!["apple"]);
        assert!(result.missed_patterns.is_empty());
    }

    #[test]
    fn test_student_has_more_matches() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["one match"]);
        let student_lines = to_string_vec(&["one match", "two matches", "three matches"]);
        let pattern = "match";
        let section = mock_subsection(5);
        // memo_count = 1, student_count = 3. Student has more, so should match.
        let result = comparator.compare(&section, &memo_lines, &student_lines, pattern);
        assert_eq!(result.awarded, 5);
        assert_eq!(result.matched_patterns, vec!["match"]);
        assert!(result.missed_patterns.is_empty());
    }

    #[test]
    fn test_student_has_fewer_matches() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["correct", "correct", "correct"]);
        let student_lines = to_string_vec(&["correct", "wrong"]);
        let pattern = "correct";
        let section = mock_subsection(20);
        // memo_count = 3, student_count = 1. Student has fewer, so should not match.
        let result = comparator.compare(&section, &memo_lines, &student_lines, pattern);
        assert_eq!(result.awarded, 0);
        assert!(result.matched_patterns.is_empty());
        assert_eq!(result.missed_patterns, vec!["correct"]);
    }

    #[test]
    fn test_no_matches_in_memo() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["no expected output", "none"]);
        let student_lines = to_string_vec(&["unexpected", "unexpected"]);
        let pattern = "unexpected";
        let section = mock_subsection(10);
        // memo_count = 0. Should not match, regardless of student output.
        let result = comparator.compare(&section, &memo_lines, &student_lines, pattern);
        assert_eq!(result.awarded, 0);
        assert!(result.matched_patterns.is_empty());
        assert_eq!(result.missed_patterns, vec!["unexpected"]);
    }

    #[test]
    fn test_no_matches_in_student() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["required", "required"]);
        let student_lines = to_string_vec(&["something else"]);
        let pattern = "required";
        let section = mock_subsection(15);
        // memo_count = 2, student_count = 0. Should not match.
        let result = comparator.compare(&section, &memo_lines, &student_lines, pattern);
        assert_eq!(result.awarded, 0);
        assert!(result.matched_patterns.is_empty());
        assert_eq!(result.missed_patterns, vec!["required"]);
    }

    #[test]
    fn test_no_matches_in_either() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["empty"]);
        let student_lines = to_string_vec(&["empty"]);
        let pattern = "unique";
        let section = mock_subsection(10);
        // memo_count = 0, student_count = 0. Should not match.
        let result = comparator.compare(&section, &memo_lines, &student_lines, pattern);
        assert_eq!(result.awarded, 0);
        assert!(result.matched_patterns.is_empty());
        assert_eq!(result.missed_patterns, vec!["unique"]);
    }
} 