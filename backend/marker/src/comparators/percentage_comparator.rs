//! A comparator that awards marks based on the percentage of matching patterns, where **line order matters**.
//!
//! The `PercentageComparator` calculates the ratio of the occurrence of a pattern in the student's
//! output compared to the memo's output and awards marks proportionally. **Lines are compared in order; only lines at the same position are considered a match.**

use crate::traits::comparator::OutputComparator;
use crate::types::{TaskResult, Subsection};

/// A comparator that awards marks based on the percentage of matching lines between student and memo output.
///
/// This comparator is useful for tasks where partial credit is desirable. The final marks are
/// calculated as a percentage of `max_marks`, based on how many of the required pattern occurrences
/// are present in the student's output. **Extra lines in the student output are penalized: the score is multiplied by the ratio of memo lines to student lines if student_lines > memo_lines.**
///
/// **Note:** Line order matters. Only lines at the same index in both memo and student outputs are considered for matching.
pub struct PercentageComparator;

/// TODO: Add the ability to toggle between line order matters and line order doesn't matter.

impl OutputComparator for PercentageComparator {
    /// Compares student and memo outputs based on the percentage of matching lines.
    ///
    /// # Arguments
    ///
    /// * `section` - The subsection entry with name and value.
    /// * `memo_lines` - The lines from the memo output.
    /// * `student_lines` - The lines from the student's output.
    ///
    /// # Returns
    ///
    /// A `TaskResult` with marks awarded based on the percentage of matching lines
    /// between the student and memo outputs.
    fn compare(
        &self,
        section: &Subsection,
        memo_lines: &[String],
        student_lines: &[String],
    ) -> TaskResult {
        if memo_lines.is_empty() {
            return TaskResult {
                name: section.name.clone(),
                awarded: if student_lines.is_empty() { section.value } else { 0 },
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

        let mut matched_count = 0;
        let mut matched_patterns = Vec::new();
        let mut missed_patterns = Vec::new();
        let min_len = memo_lines.len().min(student_lines.len());

        for i in 0..min_len {
            if memo_lines[i] == student_lines[i] {
                matched_count += 1;
                matched_patterns.push(memo_lines[i].clone());
            } else {
                missed_patterns.push(memo_lines[i].clone());
            }
        }
  
        for i in min_len..memo_lines.len() {
            missed_patterns.push(memo_lines[i].clone());
        }

        let percentage = matched_count as f32 / memo_lines.len() as f32;
        let mut awarded = (section.value as f32 * percentage).round() as i64;

        if student_lines.len() > memo_lines.len() && student_lines.len() > 0 {
            let penalty = memo_lines.len() as f32 / student_lines.len() as f32;
            awarded = (awarded as f32 * penalty).round() as i64;
        }

        TaskResult {
            name: section.name.clone(),
            awarded: awarded as i64,
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
    fn test_perfect_match_percentage() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["line 1", "line 2"]);
        let student_lines = to_string_vec(&["line 1", "line 2"]);
        let section = mock_subsection(10);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 10);
        assert_eq!(result.matched_patterns.len(), 2);
        assert!(result.missed_patterns.is_empty());
    }

    #[test]
    fn test_partial_match_percentage() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["line 1", "line 2", "line 3", "line 4"]);
        let student_lines = to_string_vec(&["line 1", "line 2"]);
        let section = mock_subsection(20);
        // 2 out of 4 lines match (50%), so 10 marks should be awarded.
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 10);
        assert_eq!(result.matched_patterns.len(), 2);
        assert_eq!(result.missed_patterns.len(), 2);
    }

    #[test]
    fn test_no_match_percentage() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["a", "b", "c"]);
        let student_lines = to_string_vec(&["d", "e", "f"]);
        let section = mock_subsection(15);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 0);
        assert!(result.matched_patterns.is_empty());
        assert_eq!(result.missed_patterns.len(), 3);
    }

    #[test]
    fn test_empty_inputs_percentage() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&[]);
        let student_lines = to_string_vec(&[]);
        let section = mock_subsection(5);
        // Both empty, should be a 100% match.
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 5);
    }

    #[test]
    fn test_empty_memo_non_empty_student_percentage() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&[]);
        let student_lines = to_string_vec(&["extra"]);
        let section = mock_subsection(5);
        // Memo is empty, student is not. This is a 0% match.
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 0);
    }

    #[test]
    fn test_non_empty_memo_empty_student_percentage() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["required"]);
        let student_lines = to_string_vec(&[]);
        let section = mock_subsection(5);
        // 0% match.
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 0);
    }

    #[test]
    fn test_duplicate_lines_in_memo() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["a", "b", "a"]);
        let student_lines = to_string_vec(&["a", "b"]);
        let section = mock_subsection(12);
        // 2 out of 3 lines match (66.6%), so 8 marks.
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 8);
    }

    #[test]
    fn test_duplicate_lines_in_student() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["a", "b"]);
        let student_lines = to_string_vec(&["a", "b", "a", "b"]);
        let section = mock_subsection(10);
        // All memo lines are found, but extra lines are penalised, so 50% match.
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 5);
    }

    #[test]
    fn test_extra_lines_penalized() {
        let comparator = PercentageComparator;
        let memo_lines = to_string_vec(&["a", "b"]);
        let student_lines = to_string_vec(&["a", "b", "extra"]);
        let section = mock_subsection(10);
        // All memo lines are found, but there is an extra line, so the score should be less than 10.
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert!(result.awarded < 10);
        assert!(result.awarded > 0);
    }
} 