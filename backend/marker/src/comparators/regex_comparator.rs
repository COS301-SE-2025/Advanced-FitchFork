//! A comparator that uses regular expressions to find matches and award marks proportionally, where **line order matters**.
//!
//! The `RegexComparator` is a flexible tool that allows for pattern matching using regex.
//! It calculates the ratio of matches found in the student's output against the memo's output
//! and awards marks based on this percentage. **Lines are compared in order; only lines at the same position are considered a match.**

use crate::traits::comparator::OutputComparator;
use crate::types::{Subsection, TaskResult};
use regex::Regex;

/// A comparator that uses a regular expression to match patterns and awards marks proportionally.
///
/// This comparator is ideal for tasks where the correctness of the output can be verified with
/// a regular expression. It provides a powerful way to validate complex patterns. Marks are awarded
/// based on the ratio of matches in the student's output compared to the memo's output. **Extra lines in the student output are penalized: the score is multiplied by the ratio of memo lines to student lines if student_lines > memo_lines.**
///
/// **Note:** Line order matters. Only lines at the same index in both memo and student outputs are considered for matching.
pub struct RegexComparator;

impl OutputComparator for RegexComparator {
    /// Compares student and memo outputs using a regular expression.
    ///
    /// # Arguments
    ///
    /// * `section` - The subsection entry containing details like name and total possible value.
    /// * `memo_lines` - A slice of strings representing the regex patterns for the memo output.
    /// * `student_lines` - A slice of strings representing the lines of the student's output.
    ///
    /// # Returns
    ///
    /// Returns a `TaskResult` with marks proportional to the similarity of regex matches.
    fn compare(
        &self,
        section: &Subsection,
        memo_lines: &[String],
        student_lines: &[String],
    ) -> TaskResult {
        if memo_lines.is_empty() {
            return TaskResult {
                name: section.name.clone(),
                awarded: if student_lines.is_empty() {
                    section.value
                } else {
                    0
                },
                possible: section.value,
                matched_patterns: vec![],
                missed_patterns: vec![],
                student_output: student_lines.to_vec(),
                memo_output: memo_lines.to_vec(),
                stderr: None,
                return_code: None,
                manual_feedback: section.feedback.clone(),
            };
        }
        
        let mut awarded_marks = 0;
        let mut matched_patterns = vec![];
        let mut missed_patterns = vec![];

        for (i, pattern) in memo_lines.iter().enumerate() {
            let regex = match Regex::new(pattern) {
                Ok(re) => re,
                Err(_) => {
                    missed_patterns.push(format!("Invalid regex pattern: {}", pattern));
                    continue;
                }
            };

            if student_lines
                .get(i)
                .map_or(false, |line| regex.is_match(line))
            {
                awarded_marks += 1;
                matched_patterns.push(pattern.clone());
            } else {
                missed_patterns.push(pattern.clone());
            }
        }

        let total_patterns = memo_lines.len();
        let mut awarded = if total_patterns == 0 {
            if student_lines.is_empty() {
                section.value
            } else {
                0
            }
        } else {
            let ratio = awarded_marks as f32 / total_patterns as f32;
            (section.value as f32 * ratio).round() as i64
        };

        if student_lines.len() > memo_lines.len() && student_lines.len() > 0 {
            let penalty = memo_lines.len() as f32 / student_lines.len() as f32;
            awarded = (awarded as f32 * penalty).round() as i64;
        }

        TaskResult {
            name: section.name.clone(),
            awarded,
            possible: section.value,
            matched_patterns,
            missed_patterns,
            student_output: student_lines.to_vec(),
            memo_output: memo_lines.to_vec(),
            stderr: None,
            return_code: None,
            manual_feedback: section.feedback.clone(),
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

    fn mock_subsection(value: i64) -> Subsection {
        Subsection {
            name: "Mock Subsection".to_string(),
            value,
            feedback: None,
            regex: None,
        }
    }

    #[test]
    fn test_perfect_match_with_regex() {
        let comparator = RegexComparator;
        let memo_lines = to_string_vec(&[r"number: \d+", r"item: \w+"]);
        let student_lines = to_string_vec(&["number: 123", "item: abc"]);
        let section = mock_subsection(10);

        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 10);
    }

    #[test]
    fn test_partial_match_with_regex() {
        let comparator = RegexComparator;
        let memo_lines = to_string_vec(&[r"apple", r"banana"]);
        let student_lines = to_string_vec(&["apple", "orange"]);
        let section = mock_subsection(20);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 10);
    }

    #[test]
    fn test_more_student_matches_than_memo() {
        let comparator = RegexComparator;
        let memo_lines = to_string_vec(&[r"item-\d"]);
        let student_lines = to_string_vec(&["item-1", "item-2", "item-3"]);
        let section = mock_subsection(5);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 2);
    }

    #[test]
    fn test_no_student_matches_with_regex() {
        let comparator = RegexComparator;
        let memo_lines = to_string_vec(&[r"email: \S+@\S+\.\S+"]);
        let student_lines = to_string_vec(&["not an email", "another one"]);
        let section = mock_subsection(15);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 0);
    }

    #[test]
    fn test_no_memo_matches_but_student_has_matches() {
        let comparator = RegexComparator;
        let memo_lines = to_string_vec(&[r"pattern-that-matches-nothing"]);
        let student_lines = to_string_vec(&["123", "456"]);
        let section = mock_subsection(10);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 0);
    }

    #[test]
    fn test_no_matches_in_memo_or_student() {
        let comparator = RegexComparator;
        let memo_lines = to_string_vec(&[r"pattern-that-matches-nothing"]);
        let student_lines = to_string_vec(&["abc", "def"]);
        let section = mock_subsection(10);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 0);
    }

    #[test]
    fn test_invalid_regex_pattern() {
        let comparator = RegexComparator;
        let memo_lines = to_string_vec(&[r"[", r"valid-pattern"]); // Invalid and valid regex
        let student_lines = to_string_vec(&["some content", "valid-pattern"]);
        let section = mock_subsection(10);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 5);
        assert!(!result.missed_patterns.is_empty());
        assert!(
            result
                .missed_patterns
                .iter()
                .any(|p| p.starts_with("Invalid regex pattern"))
        );
    }

    #[test]
    fn test_multiple_matches_in_one_line() {
        let comparator = RegexComparator;
        let memo_lines = to_string_vec(&[r"tag\d"]);
        let student_lines = to_string_vec(&["tag1 tag2"]);
        let section = mock_subsection(10);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 10);
    }

    #[test]
    fn test_extra_lines_penalized() {
        let comparator = RegexComparator;
        let memo_lines = to_string_vec(&[r"^a$", r"^b$"]);
        let student_lines = to_string_vec(&["a", "b", "extra"]);
        let section = mock_subsection(10);
        // All memo patterns are matched, but there is an extra line, so the score should be less than 10.
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert!(result.awarded < 10);
        assert!(result.awarded > 0);
    }
}
