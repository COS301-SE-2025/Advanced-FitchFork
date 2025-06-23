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
/// **Extra lines in the student output are penalized: full marks are only awarded if the number of lines matches exactly.**
pub struct ExactComparator;

impl OutputComparator for ExactComparator {
    /// Compares student and memo outputs for an exact, line-by-line match.
    ///
    /// # Arguments
    ///
    /// * `section` - The subsection entry from the allocator, containing the name and value.
    /// * `memo_lines` - The lines from the memo output file.
    /// * `student_lines` - The lines from the student's output file.
    ///
    /// # Returns
    ///
    /// Returns a `TaskResult` indicating whether the outputs were an exact match. If they match,
    /// the full value of the section is awarded. Otherwise, 0 is awarded.
    fn compare(
        &self,
        section: &Subsection,
        memo_lines: &[String],
        student_lines: &[String],
    ) -> TaskResult {
        let mut matched_patterns = Vec::new();
        let mut missed_patterns = Vec::new();

        let all_match = memo_lines.iter().all(|memo_line| {
            let found = student_lines.contains(memo_line);
            if found {
                matched_patterns.push(memo_line.clone());
            } else {
                missed_patterns.push(memo_line.clone());
            }
            found
        });

        let awarded = if all_match && memo_lines.len() == student_lines.len() {
            section.value
        } else {
            0
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
    fn test_exact_match() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["line 1", "line 2"]);
        let student_lines = to_string_vec(&["line 1", "line 2"]);
        let section = mock_subsection(10);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 10);
        assert!(result.missed_patterns.is_empty());
    }

    #[test]
    fn test_mismatched_content() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["line 1", "line 2"]);
        let student_lines = to_string_vec(&["line 1", "line 3"]);
        let section = mock_subsection(10);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 0);
        assert_eq!(result.missed_patterns, vec!["line 2"]);
    }

    #[test]
    fn test_mismatched_length() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["line 1", "line 2"]);
        let student_lines = to_string_vec(&["line 1"]);
        let section = mock_subsection(10);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 0);
        assert_eq!(result.missed_patterns, vec!["line 2"]);
    }

    #[test]
    fn test_empty_inputs_match() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&[]);
        let student_lines = to_string_vec(&[]);
        let section = mock_subsection(5);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 5);
        assert!(result.matched_patterns.is_empty());
        assert!(result.missed_patterns.is_empty());
    }

    #[test]
    fn test_empty_memo_non_empty_student() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&[]);
        let student_lines = to_string_vec(&["extra line"]);
        let section = mock_subsection(5);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 0);
    }

    #[test]
    fn test_non_empty_memo_empty_student() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["required line"]);
        let student_lines = to_string_vec(&[]);
        let section = mock_subsection(5);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 0);
        assert_eq!(result.missed_patterns, vec!["required line"]);
    }

    #[test]
    fn test_extra_lines_penalized() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["a", "b"]);
        let student_lines = to_string_vec(&["a", "b", "extra"]);
        let section = mock_subsection(10);
        // Extra line should result in 0 marks.
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 0);
    }
} 