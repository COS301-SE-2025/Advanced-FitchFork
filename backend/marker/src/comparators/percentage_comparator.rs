//! A comparator that awards marks based on the percentage of matching patterns.
//!
//! The `PercentageComparator` calculates the ratio of the occurrence of a pattern in the student's
//! output compared to the memo's output and awards marks proportionally.

use crate::traits::comparator::OutputComparator;
use crate::types::{TaskResult, Subsection};

/// A comparator that awards marks based on the percentage of matching lines between student and memo output.
///
/// This comparator is useful for tasks where partial credit is desirable. The final marks are
/// calculated as a percentage of `max_marks`, based on how many of the required pattern occurrences
/// are present in the student's output.
pub struct PercentageComparator;

impl OutputComparator for PercentageComparator {
    /// Compares student output to memo output and awards marks based on the similarity of pattern counts.
    ///
    /// Marks are awarded based on the ratio of pattern occurrences. The comparator penalizes for both
    /// missing and extra occurrences relative to the memo. For example, having half the expected
    /// occurrences gives the same score as having twice the expected occurrences.
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
    /// Returns a `TaskResult` with marks proportional to the similarity.
    fn compare(
        &self,
        section: &Subsection,
        memo_lines: &[String],
        student_lines: &[String],
        pattern: &str,
    ) -> TaskResult {
        let memo_count = memo_lines.iter().filter(|l| l.contains(pattern)).count();
        let student_count = student_lines.iter().filter(|l| l.contains(pattern)).count();

        let awarded = if memo_count == 0 {
            if student_count == 0 {
                section.value
            } else {
                0
            }
        } else {
            let ratio = if student_count > memo_count {
                memo_count as f32 / student_count as f32
            } else {
                student_count as f32 / memo_count as f32
            };

            (section.value as f32 * ratio).round() as u32
        };

        let mut matched_patterns = vec![];
        if student_count > 0 && memo_count > 0 {
            matched_patterns.push(pattern.to_string());
        }

        let mut missed_patterns = vec![];
        if student_count < memo_count || (memo_count == 0 && student_count > 0) {
            missed_patterns.push(pattern.to_string());
        }

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
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["apple", "orange", "apple"]);
        let student_lines = to_string_vec(&["apple", "apple", "grape"]);
        let pattern = "apple";
        let section = mock_subsection(10);
        // memo_count = 2, student_count = 2. percent = 1.0. marks = 10 * 1.0 = 10
        let result = comparator.compare(&section, &memo_lines, &student_lines, pattern);
        assert_eq!(result.awarded, 10);
        assert_eq!(result.matched_patterns, vec!["apple"]);
        assert!(result.missed_patterns.is_empty());
    }

    #[test]
    fn test_partial_match() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["line 1 correct", "line 2 correct", "line 3 correct", "line 4 correct"]);
        let student_lines = to_string_vec(&["line 1 correct", "line 2 wrong", "line 3 correct", "line 4 wrong"]);
        let pattern = "correct";
        let section = mock_subsection(20);
        // memo_count = 4, student_count = 2. percent = 0.5. marks = 20 * 0.5 = 10
        let result = comparator.compare(&section, &memo_lines, &student_lines, pattern);
        assert_eq!(result.awarded, 10);
        assert_eq!(result.matched_patterns, vec!["correct"]);
        assert_eq!(result.missed_patterns, vec!["correct"]);
    }

    #[test]
    fn test_more_student_matches_than_memo() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["one match"]);
        let student_lines = to_string_vec(&["one match", "two matches", "three matches"]);
        let pattern = "match";
        let section = mock_subsection(5);
        // memo_count = 1, student_count = 3. ratio = 1/3. marks = 5 * 0.333... = 1.66... rounded to 2.
        let result = comparator.compare(&section, &memo_lines, &student_lines, pattern);
        assert_eq!(result.awarded, 2);
        assert_eq!(result.matched_patterns, vec!["match"]);
        assert!(result.missed_patterns.is_empty());
    }

    #[test]
    fn test_no_student_matches() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["hello", "world", "hello"]);
        let student_lines = to_string_vec(&["goodbye", "world"]);
        let pattern = "hello";
        let section = mock_subsection(15);
        // memo_count = 2, student_count = 0. percent = 0.0. marks = 15 * 0.0 = 0
        let result = comparator.compare(&section, &memo_lines, &student_lines, pattern);
        assert_eq!(result.awarded, 0);
        assert!(result.matched_patterns.is_empty());
        assert_eq!(result.missed_patterns, vec!["hello"]);
    }

    #[test]
    fn test_no_memo_matches() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["no matches here", "or here"]);
        let student_lines = to_string_vec(&["pattern", "pattern", "pattern"]);
        let pattern = "pattern";
        let section = mock_subsection(10);
        // memo_count = 0, student_count = 3. Should return 0.
        let result = comparator.compare(&section, &memo_lines, &student_lines, pattern);
        assert_eq!(result.awarded, 0);
        assert!(result.matched_patterns.is_empty());
        assert_eq!(result.missed_patterns, vec!["pattern"]);
    }

    #[test]
    fn test_no_memo_no_student_matches() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["nothing here"]);
        let student_lines = to_string_vec(&["or here"]);
        let pattern = "unique";
        let section = mock_subsection(10);
        // memo_count = 0, student_count = 0. Should return max_marks.
        let result = comparator.compare(&section, &memo_lines, &student_lines, pattern);
        assert_eq!(result.awarded, 10);
        assert!(result.matched_patterns.is_empty());
        assert!(result.missed_patterns.is_empty());
    }

    #[test]
    fn test_rounding_logic() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["a", "a", "a"]);
        let student_lines_half_round_down = to_string_vec(&["a"]);
        let pattern = "a";
        let section = mock_subsection(10);
        // memo_count = 3, student_count = 1. percent = 1/3 = 0.333...
        // marks = 10 * 0.333... = 3.333... rounded = 3
        let result1 = comparator.compare(&section, &memo_lines, &student_lines_half_round_down, pattern);
        assert_eq!(result1.awarded, 3);

        let student_lines_half_round_up = to_string_vec(&["a", "a"]);
        // memo_count = 3, student_count = 2. percent = 2/3 = 0.666...
        // marks = 10 * 0.666... = 6.666... rounded = 7
        let result2 = comparator.compare(&section, &memo_lines, &student_lines_half_round_up, pattern);
        assert_eq!(result2.awarded, 7);
    }
} 