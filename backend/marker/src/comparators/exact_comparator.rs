//! A comparator that performs an exact match comparison between memo and student output, where **line order matters**.
//!
//! The `ExactComparator` is designed to award marks on an all-or-nothing basis. It checks if a
//! specific pattern appears in the student's output at the same position as in the memo (solution) output. **Lines are compared in order; only lines at the same position are considered a match.**

use crate::traits::comparator::OutputComparator;
use crate::types::TaskResult;
use util::mark_allocator::Subsection;

/// A comparator that awards full marks if the student's output matches the memo output exactly, line by line and in order.
///
/// This comparator is useful for tasks where the presence, frequency, and order of a specific output line
/// or pattern is a critical success factor. If the expected pattern appears at the same position in the memo and student output, full marks are awarded. **Extra lines in the student output are penalized: full marks are only awarded if the number of lines matches exactly.**
///
/// **Note:** Line order matters. Only lines at the same index in both memo and student outputs are considered for matching.
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

        // Different lengths => immediate fail. Record memo lines that are missing.
        if memo_lines.len() != student_lines.len() {
            for i in student_lines.len()..memo_lines.len() {
                missed_patterns.push(memo_lines[i].clone());
            }
            return TaskResult {
                name: section.name.clone(),
                awarded: 0.0,
                possible: section.value,
                matched_patterns,
                missed_patterns,
                student_output: student_lines.to_vec(),
                memo_output: memo_lines.to_vec(),
                stderr: None,
                return_code: None,
                manual_feedback: section.feedback.clone(),
            };
        }

        let mut all_match = true;
        for (i, memo_line) in memo_lines.iter().enumerate() {
            if student_lines[i] == *memo_line {
                matched_patterns.push(memo_line.clone());
            } else {
                missed_patterns.push(memo_line.clone());
                all_match = false;
            }
        }

        let awarded = if all_match { section.value } else { 0.0 };

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
    use util::mark_allocator::Subsection;

    /// Helper: make Vec<String> from &str slice.
    fn to_string_vec(lines: &[&str]) -> Vec<String> {
        lines.iter().map(|s| s.to_string()).collect()
    }

    fn mock_subsection(value: f64) -> Subsection {
        Subsection {
            name: "Mock Subsection".to_string(),
            value,
            regex: None,
            feedback: None,
        }
    }

    #[test]
    fn test_exact_match() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["line 1", "line 2"]);
        let student_lines = to_string_vec(&["line 1", "line 2"]);
        let section = mock_subsection(10.0);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 10.0);
        assert!(result.missed_patterns.is_empty());
    }

    #[test]
    fn test_mismatched_content() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["line 1", "line 2"]);
        let student_lines = to_string_vec(&["line 1", "line 3"]);
        let section = mock_subsection(10.0);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 0.0);
        assert_eq!(result.missed_patterns, vec!["line 2"]);
    }

    #[test]
    fn test_mismatched_length() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["line 1", "line 2"]);
        let student_lines = to_string_vec(&["line 1"]);
        let section = mock_subsection(10.0);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 0.0);
        assert_eq!(result.missed_patterns, vec!["line 2"]);
    }

    #[test]
    fn test_empty_inputs_match() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&[]);
        let student_lines = to_string_vec(&[]);
        let section = mock_subsection(5.0);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 5.0);
        assert!(result.matched_patterns.is_empty());
        assert!(result.missed_patterns.is_empty());
    }

    #[test]
    fn test_empty_memo_non_empty_student() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&[]);
        let student_lines = to_string_vec(&["extra line"]);
        let section = mock_subsection(5.0);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 0.0);
    }

    #[test]
    fn test_non_empty_memo_empty_student() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["required line"]);
        let student_lines = to_string_vec(&[]);
        let section = mock_subsection(5.0);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 0.0);
        assert_eq!(result.missed_patterns, vec!["required line"]);
    }

    #[test]
    fn test_extra_lines_penalized() {
        let comparator = ExactComparator;
        let memo_lines = to_string_vec(&["a", "b"]);
        let student_lines = to_string_vec(&["a", "b", "extra"]);
        let section = mock_subsection(10.0);
        let result = comparator.compare(&section, &memo_lines, &student_lines);
        assert_eq!(result.awarded, 0.0);
    }
}
