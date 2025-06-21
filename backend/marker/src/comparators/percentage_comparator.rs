//! A comparator that awards marks based on the percentage of matching patterns.
//!
//! The `PercentageComparator` calculates the ratio of the occurrence of a pattern in the student's
//! output compared to the memo's output and awards marks proportionally.

use crate::traits::comparator::OutputComparator;

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
    /// * `memo_lines` - A slice of strings representing the lines of the memo output.
    /// * `student_lines` - A slice of strings representing the lines of the student's output.
    /// * `pattern` - The exact string pattern to search for in the output lines.
    /// * `max_marks` - The maximum marks available for this comparison.
    ///
    /// # Returns
    ///
    /// Returns a number of marks proportional to the similarity.
    /// If `memo_count` is 0, it returns `max_marks` if `student_count` is also 0; otherwise, it returns 0.
    fn compare(&self, memo_lines: &[String], student_lines: &[String], pattern: &str, max_marks: u32) -> u32 {
        let memo_count = memo_lines.iter().filter(|l| l.contains(pattern)).count();
        let student_count = student_lines.iter().filter(|l| l.contains(pattern)).count();

        if memo_count == 0 {
            return if student_count == 0 { max_marks } else { 0 };
        }

        let ratio = if student_count > memo_count {
            memo_count as f32 / student_count as f32
        } else {
            student_count as f32 / memo_count as f32
        };

        (max_marks as f32 * ratio).round() as u32
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
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["apple", "orange", "apple"]);
        let student_lines = to_string_vec(&["apple", "apple", "grape"]);
        let pattern = "apple";
        let max_marks = 10;
        // memo_count = 2, student_count = 2. percent = 1.0. marks = 10 * 1.0 = 10
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 10);
    }

    #[test]
    fn test_partial_match() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["line 1 correct", "line 2 correct", "line 3 correct", "line 4 correct"]);
        let student_lines = to_string_vec(&["line 1 correct", "line 2 wrong", "line 3 correct", "line 4 wrong"]);
        let pattern = "correct";
        let max_marks = 20;
        // memo_count = 4, student_count = 2. percent = 0.5. marks = 20 * 0.5 = 10
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 10);
    }

    #[test]
    fn test_more_student_matches_than_memo() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["one match"]);
        let student_lines = to_string_vec(&["one match", "two matches", "three matches"]);
        let pattern = "match";
        let max_marks = 5;
        // memo_count = 1, student_count = 3. ratio = 1/3. marks = 5 * 0.333... = 1.66... rounded to 2.
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 2);
    }

    #[test]
    fn test_no_student_matches() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["hello", "world", "hello"]);
        let student_lines = to_string_vec(&["goodbye", "world"]);
        let pattern = "hello";
        let max_marks = 15;
        // memo_count = 2, student_count = 0. percent = 0.0. marks = 15 * 0.0 = 0
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 0);
    }

    #[test]
    fn test_no_memo_matches() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["no matches here", "or here"]);
        let student_lines = to_string_vec(&["pattern", "pattern", "pattern"]);
        let pattern = "pattern";
        let max_marks = 10;
        // memo_count = 0, student_count = 3. Should return 0.
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 0);
    }

    #[test]
    fn test_no_memo_no_student_matches() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["nothing here"]);
        let student_lines = to_string_vec(&["or here"]);
        let pattern = "unique";
        let max_marks = 10;
        // memo_count = 0, student_count = 0. Should return max_marks.
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 10);
    }

    #[test]
    fn test_rounding_logic() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["a", "a", "a"]);
        let student_lines_half_round_down = to_string_vec(&["a"]);
        let pattern = "a";
        let max_marks = 10;
        // memo_count = 3, student_count = 1. percent = 1/3 = 0.333...
        // marks = 10 * 0.333... = 3.333... rounded = 3
        assert_eq!(comparator.compare(&memo_lines, &student_lines_half_round_down, pattern, max_marks), 3);

        let student_lines_half_round_up = to_string_vec(&["a", "a"]);
        // memo_count = 3, student_count = 2. percent = 2/3 = 0.666...
        // marks = 10 * 0.666... = 6.666... rounded = 7
        assert_eq!(comparator.compare(&memo_lines, &student_lines_half_round_up, pattern, max_marks), 7);
    }
} 