//! A comparator that uses regular expressions to find matches and award marks proportionally.
//!
//! The `RegexComparator` is a flexible tool that allows for pattern matching using regex.
//! It calculates the ratio of matches found in the student's output against the memo's output
//! and awards marks based on this percentage.

use crate::traits::comparator::OutputComparator;
use regex::Regex;

/// A comparator that uses a regular expression to match patterns and awards marks proportionally.
///
/// This comparator is ideal for tasks where the correctness of the output can be verified with
/// a regular expression. It provides a powerful way to validate complex patterns. Marks are awarded
/// based on the ratio of matches in the student's output compared to the memo's output.
pub struct RegexComparator;

impl OutputComparator for RegexComparator {
    /// Compares student and memo outputs using a regular expression.
    ///
    /// # Arguments
    ///
    /// * `memo_lines` - A slice of strings representing the lines of the memo output.
    /// * `student_lines` - A slice of strings representing the lines of the student's output.
    /// * `pattern` - The regular expression pattern to search for. If the pattern is invalid,
    ///   0 marks will be awarded.
    /// * `max_marks` - The maximum marks available for this comparison.
    ///
    /// # Returns
    ///
    /// Returns marks proportional to the similarity of regex matches. If the pattern is invalid,
    /// 0 marks are awarded. If `memo_matches` is 0, it returns `max_marks` if `student_matches`
    /// is also 0; otherwise, it returns 0.
    fn compare(&self, memo_lines: &[String], student_lines: &[String], pattern: &str, max_marks: u32) -> u32 {
        let regex = match Regex::new(pattern) {
            Ok(re) => re,
            Err(_) => return 0, // Invalid regex pattern
        };

        let memo_matches = memo_lines.iter().flat_map(|line| regex.find_iter(line)).count();
        let student_matches = student_lines.iter().flat_map(|line| regex.find_iter(line)).count();

        if memo_matches == 0 {
            return if student_matches == 0 { max_marks } else { 0 };
        }

        let ratio = if student_matches > memo_matches {
            memo_matches as f32 / student_matches as f32
        } else {
            student_matches as f32 / memo_matches as f32
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
    fn test_perfect_match_with_regex() {
        let comparator = RegexComparator;
        let memo_lines = to_string_vec(&["number: 123", "number: 456"]);
        let student_lines = to_string_vec(&["number: 789", "number: 012"]);
        let pattern = r"number: \d+";
        let max_marks = 10;
        // memo_matches = 2, student_matches = 2. ratio = 1.0. marks = 10.
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 10);
    }

    #[test]
    fn test_partial_match_with_regex() {
        let comparator = RegexComparator;
        let memo_lines = to_string_vec(&["word", "word", "word", "word"]);
        let student_lines = to_string_vec(&["word", "nope", "word", "nope"]);
        let pattern = r"word";
        let max_marks = 20;
        // memo_matches = 4, student_matches = 2. ratio = 0.5. marks = 10.
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 10);
    }

    #[test]
    fn test_more_student_matches_than_memo() {
        let comparator = RegexComparator;
        let memo_lines = to_string_vec(&["item-1"]);
        let student_lines = to_string_vec(&["item-1", "item-2", "item-3"]);
        let pattern = r"item-\d";
        let max_marks = 5;
        // memo_matches = 1, student_matches = 3. ratio = 1/3. marks = 5 * 0.333... = 2.
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 2);
    }

    #[test]
    fn test_no_student_matches_with_regex() {
        let comparator = RegexComparator;
        let memo_lines = to_string_vec(&["email: a@b.com", "email: c@d.com"]);
        let student_lines = to_string_vec(&["not an email", "another one"]);
        let pattern = r"email: \S+@\S+\.\S+";
        let max_marks = 15;
        // memo_matches = 2, student_matches = 0. ratio = 0.0. marks = 0.
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 0);
    }

    #[test]
    fn test_no_memo_matches_but_student_has_matches() {
        let comparator = RegexComparator;
        let memo_lines = to_string_vec(&["nothing here"]);
        let student_lines = to_string_vec(&["123", "456"]);
        let pattern = r"\d+";
        let max_marks = 10;
        // memo_matches = 0, student_matches = 2. Should return 0.
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 0);
    }

    #[test]
    fn test_no_matches_in_memo_or_student() {
        let comparator = RegexComparator;
        let memo_lines = to_string_vec(&["nothing here"]);
        let student_lines = to_string_vec(&["abc", "def"]);
        let pattern = r"\d+";
        let max_marks = 10;
        // memo_matches = 0, student_matches = 0. Should return max_marks.
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 10);
    }

    #[test]
    fn test_invalid_regex_pattern() {
        let comparator = RegexComparator;
        let memo_lines = to_string_vec(&["some content"]);
        let student_lines = to_string_vec(&["some content"]);
        let pattern = r"["; // Invalid regex
        let max_marks = 10;
        // Invalid regex should result in 0 marks.
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 0);
    }

    #[test]
    fn test_multiple_matches_in_one_line() {
        let comparator = RegexComparator;
        let memo_lines = to_string_vec(&["tag1 tag2"]);
        let student_lines = to_string_vec(&["tag1"]);
        let pattern = r"tag\d";
        let max_marks = 10;
        // memo_matches = 2, student_matches = 1. ratio = 0.5. marks = 5.
        assert_eq!(comparator.compare(&memo_lines, &student_lines, pattern, max_marks), 5);
    }
} 