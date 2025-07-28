//!
//! Output Parser Module
//!
//! This module provides functionality to parse output content (memo and student) into structured
//! submissions with tasks and subtasks. It handles the conversion from raw text content with delimiters
//! into a structured in-memory representation that can be used by comparators.
//!
//! # Functionality
//!
//! - Parses output content containing concatenated subtasks with delimiters
//! - Extracts task and subtask structures from raw text
//! - Validates that the number of subtasks matches expected counts
//! - Groups subtasks into tasks based on allocator schema
//! - Each file pair (memo/student) represents a task within the submission
//!
//! # Error Handling
//!
//! Returns [`MarkerError`] variants for invalid output format or mismatched counts.

use crate::error::MarkerError;
use crate::traits::parser::Parser;
use regex::Regex;
use regex::escape;
use util::execution_config::ExecutionConfig;

/// Represents a parsed submission containing multiple tasks.
#[derive(Debug)]
pub struct Submission {
    /// List of tasks extracted from the submission files.
    pub tasks: Vec<Task>,
}

/// Represents a single task within a submission.
#[derive(Debug)]
pub struct Task {
    /// Task identifier (e.g., "Task1").
    pub task_id: String,
    /// Memo output for this task.
    pub memo_output: TaskOutput,
    /// Student output for this task.
    pub student_output: TaskOutput,
}

/// Represents the output content for a single task.
#[derive(Debug)]
pub struct TaskOutput {
    /// List of subtasks within this task.
    pub subtasks: Vec<SubtaskOutput>,
}

/// Represents a single subtask within a task.
#[derive(Debug)]
pub struct SubtaskOutput {
    /// Subtask name (e.g., "Task1Subtask2").
    pub name: String,
    /// Lines of output for this subtask.
    pub lines: Vec<String>,
}

pub struct OutputParser;

impl<'a> Parser<(&'a [String], &'a [String], Vec<usize>), Submission> for OutputParser {
    fn parse(
        &self,
        input: (&'a [String], &'a [String], Vec<usize>),
        config: ExecutionConfig,
    ) -> Result<Submission, MarkerError> {
        let (memo_contents, student_contents, expected_subtasks) = input;
        if memo_contents.len() != student_contents.len() {
            return Err(MarkerError::ParseOutputError(format!(
                "Number of memo files ({}) does not match number of student files ({})",
                memo_contents.len(),
                student_contents.len()
            )));
        }

        if memo_contents.len() != expected_subtasks.len() {
            return Err(MarkerError::ParseOutputError(format!(
                "Number of tasks ({}) does not match the number of expected subtask counts ({})",
                memo_contents.len(),
                expected_subtasks.len()
            )));
        }

        let mut tasks = Vec::new();
        for (i, (memo_content, student_content)) in memo_contents
            .iter()
            .zip(student_contents.iter())
            .enumerate()
        {
            let task_id = format!("Task{}", i + 1);
            let expected_subtask_count = expected_subtasks[i];
            let memo_output = parse_task_output(memo_content, expected_subtask_count, &config)?;
            let student_output =
                parse_task_output(student_content, expected_subtask_count, &config)?;

            tasks.push(Task {
                task_id,
                memo_output,
                student_output,
            });
        }

        Ok(Submission { tasks })
    }
}

/// Parse a single task's output content into structured subtasks.
///
/// # Arguments
///
/// * `content` - Raw content of the task output file
/// * `expected_subtask_count` - Expected number of subtasks in this task
///
/// # Returns
///
/// A [`TaskOutput`] struct containing the parsed subtasks.
///
/// # Errors
///
/// Returns [`MarkerError`] for invalid output format or mismatched counts.
fn parse_task_output(
    content: &str,
    expected_subtask_count: usize,
    config: &ExecutionConfig,
) -> Result<TaskOutput, MarkerError> {
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    if lines.is_empty() {
        return Err(MarkerError::ParseOutputError(
            "Content is empty".to_string(),
        ));
    }

    let content_lines = &lines[1..];
    let deliminator = config.deliminator.clone();
    let pattern = format!(r"^{}(.+)$", escape(&deliminator));
    let delimiter_regex = Regex::new(&pattern)
        .map_err(|e| MarkerError::ParseOutputError(format!("Failed to compile regex: {}", e)))?;

    //No longer hardcoded
    // let delimiter_regex = Regex::new(r"^&-=-&(.+)$")
    //     .map_err(|e| MarkerError::ParseOutputError(format!("Failed to compile regex: {}", e)))?;

    let mut delimiters = Vec::new();
    for (line_num, line) in content_lines.iter().enumerate() {
        if let Some(captures) = delimiter_regex.captures(line) {
            if let Some(subtask_name) = captures.get(1) {
                delimiters.push((line_num, subtask_name.as_str().to_string()));
            }
        }
    }

    if delimiters.len() != expected_subtask_count {
        return Err(MarkerError::ParseOutputError(format!(
            "Expected {} subtasks, but found {} delimiters",
            expected_subtask_count,
            delimiters.len()
        )));
    }

    let mut subtasks = Vec::new();
    for (i, (line_num, subtask_name)) in delimiters.iter().enumerate() {
        let start_line = *line_num + 1;
        let end_line = if i + 1 < delimiters.len() {
            delimiters[i + 1].0
        } else {
            content_lines.len()
        };

        let subtask_lines: Vec<String> = content_lines[start_line..end_line]
            .iter()
            .map(|s| s.to_string())
            .collect();

        subtasks.push(SubtaskOutput {
            name: subtask_name.clone(),
            lines: subtask_lines,
        });
    }

    Ok(TaskOutput { subtasks })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_task_output_valid_format() {
        let content = r#"gcc -o program program.c
&-=-&Subtask1
line1
line2
&-=-&Subtask2
line3"#;
        let result = parse_task_output(content, 2, &ExecutionConfig::default_config());
        assert!(result.is_ok());
        let task_output = result.unwrap();
        assert_eq!(task_output.subtasks.len(), 2);
        assert_eq!(task_output.subtasks[0].name, "Subtask1");
        assert_eq!(task_output.subtasks[0].lines, vec!["line1", "line2"]);
        assert_eq!(task_output.subtasks[1].name, "Subtask2");
        assert_eq!(task_output.subtasks[1].lines, vec!["line3"]);
    }

    #[test]
    fn test_parse_task_output_too_few_delimiters() {
        let content = r#"gcc -o program program.c
&-=-&Subtask1
line1"#;
        let result = parse_task_output(content, 2, &ExecutionConfig::default_config());
        assert!(result.is_err());
        match result.err().unwrap() {
            MarkerError::ParseOutputError(msg) => {
                assert!(msg.contains("Expected 2 subtasks, but found 1 delimiters"));
            }
            _ => panic!("Expected InvalidOutputFormat error"),
        }
    }

    #[test]
    fn test_parse_task_output_too_many_delimiters() {
        let content = r#"gcc -o program program.c
&-=-&Subtask1
line1
&-=-&Subtask2
line2
&-=-&Subtask3
line3"#;
        let result = parse_task_output(content, 2, &ExecutionConfig::default_config());
        assert!(result.is_err());
        match result.err().unwrap() {
            MarkerError::ParseOutputError(msg) => {
                assert!(msg.contains("Expected 2 subtasks, but found 3 delimiters"));
            }
            _ => panic!("Expected InvalidOutputFormat error"),
        }
    }

    #[test]
    fn test_parse_task_output_empty_subtask_content() {
        let content = r#"gcc -o program program.c
&-=-&Subtask1
&-=-&Subtask2
line1
line2"#;
        let result = parse_task_output(content, 2, &ExecutionConfig::default_config());
        assert!(result.is_ok());
        let task_output = result.unwrap();
        assert_eq!(task_output.subtasks.len(), 2);
        assert_eq!(task_output.subtasks[0].name, "Subtask1");
        assert!(task_output.subtasks[0].lines.is_empty());
        assert_eq!(task_output.subtasks[1].name, "Subtask2");
        assert_eq!(task_output.subtasks[1].lines, vec!["line1", "line2"]);
    }

    #[test]
    fn test_parse_task_output_no_delimiters() {
        let content = r#"gcc -o program program.c
Some random output
without any delimiters."#;
        let result = parse_task_output(content, 1, &ExecutionConfig::default_config());
        assert!(result.is_err());
        match result.err().unwrap() {
            MarkerError::ParseOutputError(msg) => {
                assert!(msg.contains("Expected 1 subtasks, but found 0 delimiters"));
            }
            _ => panic!("Expected InvalidOutputFormat error"),
        }
    }

    #[test]
    fn test_parse_task_output_empty_content() {
        let content = "";
        let result = parse_task_output(content, 1, &ExecutionConfig::default_config());
        assert!(result.is_err());
        match result.err().unwrap() {
            MarkerError::ParseOutputError(msg) => {
                assert_eq!(msg, "Content is empty");
            }
            _ => panic!("Expected InvalidOutputFormat error"),
        }
    }

    #[test]
    fn test_parse_task_output_only_command_line() {
        let content = "gcc -o program program.c";
        let result = parse_task_output(content, 0, &ExecutionConfig::default_config());
        assert!(result.is_ok());
        let task_output = result.unwrap();
        assert!(task_output.subtasks.is_empty());
    }

    // Helper function to read test files
    fn read_test_file(path: &str) -> String {
        use std::fs;
        fs::read_to_string(path).expect(&format!("Failed to read test file: {}", path))
    }

    #[test]
    fn test_parse_case1_happy_path() {
        let memo_contents = vec![read_test_file(
            "src/test_files/output_parser/case1/memo.txt",
        )];
        let student_contents = vec![read_test_file(
            "src/test_files/output_parser/case1/student.txt",
        )];
        let parser = OutputParser;
        let result = parser.parse(
            (&memo_contents, &student_contents, vec![3]),
            ExecutionConfig::default_config(),
        );

        assert!(result.is_ok(), "test_parse_case1_happy_path failed");
        let submission = result.unwrap();
        assert_eq!(submission.tasks.len(), 1);
        let task = &submission.tasks[0];
        assert_eq!(task.memo_output.subtasks.len(), 3);
        assert_eq!(task.memo_output.subtasks[0].name, "Sub1");
        assert_eq!(task.memo_output.subtasks[0].lines.len(), 2);
        assert_eq!(task.memo_output.subtasks[1].name, "Sub2");
        assert_eq!(task.memo_output.subtasks[1].lines.len(), 2);
        assert_eq!(task.memo_output.subtasks[2].name, "Sub3");
        assert_eq!(task.memo_output.subtasks[2].lines.len(), 2);

        assert_eq!(task.student_output.subtasks.len(), 3);
        assert_eq!(task.student_output.subtasks[0].name, "Sub1");
        assert_eq!(task.student_output.subtasks[0].lines.len(), 2);
        assert_eq!(task.student_output.subtasks[1].name, "Sub2");
        assert_eq!(task.student_output.subtasks[1].lines.len(), 2);
        assert_eq!(task.student_output.subtasks[2].name, "Sub3");
        assert_eq!(task.student_output.subtasks[2].lines.len(), 2);
    }

    #[test]
    fn test_parse_case2_missing_delimiter() {
        let memo_contents = vec![read_test_file(
            "src/test_files/output_parser/case2/memo.txt",
        )];
        let student_contents = vec![read_test_file(
            "src/test_files/output_parser/case2/student.txt",
        )];
        let parser = OutputParser;
        let result = parser.parse(
            (&memo_contents, &student_contents, vec![3]),
            ExecutionConfig::default_config(),
        );

        match result {
            Err(MarkerError::ParseOutputError(msg)) => {
                assert!(msg.contains("Expected 3 subtasks, but found 2 delimiters"));
            }
            _ => panic!("Expected InvalidOutputFormat error for missing delimiter"),
        }
    }

    #[test]
    fn test_parse_case3_extra_delimiter() {
        let memo_contents = vec![read_test_file(
            "src/test_files/output_parser/case3/memo.txt",
        )];
        let student_contents = vec![read_test_file(
            "src/test_files/output_parser/case3/student.txt",
        )];
        let parser = OutputParser;
        let result = parser.parse(
            (&memo_contents, &student_contents, vec![3]),
            ExecutionConfig::default_config(),
        );

        match result {
            Err(MarkerError::ParseOutputError(msg)) => {
                assert!(msg.contains("Expected 3 subtasks, but found 4 delimiters"));
            }
            _ => panic!("Expected InvalidOutputFormat error for extra delimiter"),
        }
    }

    #[test]
    fn test_parse_case4_empty_subtask() {
        let memo_contents = vec![read_test_file(
            "src/test_files/output_parser/case4/memo.txt",
        )];
        let student_contents = vec![read_test_file(
            "src/test_files/output_parser/case4/student.txt",
        )];
        let parser = OutputParser;
        let result = parser.parse(
            (&memo_contents, &student_contents, vec![3]),
            ExecutionConfig::default_config(),
        );

        assert!(result.is_ok());
        let submission = result.unwrap();
        assert_eq!(submission.tasks.len(), 1);
        let task = &submission.tasks[0];
        assert_eq!(task.memo_output.subtasks.len(), 3);
        assert_eq!(task.memo_output.subtasks[0].lines.len(), 1);
        assert_eq!(task.memo_output.subtasks[1].lines.len(), 0);
        assert_eq!(task.memo_output.subtasks[2].lines.len(), 1);

        assert_eq!(task.student_output.subtasks.len(), 3);
        assert_eq!(task.student_output.subtasks[0].name, "Sub1");
        assert_eq!(task.student_output.subtasks[0].lines.len(), 1);
        assert_eq!(task.student_output.subtasks[1].name, "Sub2");
        assert_eq!(task.student_output.subtasks[1].lines.len(), 0);
        assert_eq!(task.student_output.subtasks[2].name, "Sub3");
        assert_eq!(task.student_output.subtasks[2].lines.len(), 1);
    }

    #[test]
    fn test_parse_case5_mismatched_file_counts() {
        let memo_contents = vec![read_test_file(
            "src/test_files/output_parser/case5/memo.txt",
        )];
        let student_contents = Vec::new();
        let parser = OutputParser;
        let result = parser.parse(
            (&memo_contents, &student_contents, vec![2]),
            ExecutionConfig::default_config(),
        );

        match result {
            Err(MarkerError::ParseOutputError(msg)) => {
                assert!(msg.contains(
                    "Number of memo files (1) does not match number of student files (0)"
                ));
            }
            _ => panic!("Expected InvalidOutputFormat error for mismatched file counts"),
        }
    }

    #[test]
    fn test_parse_case6_multiple_tasks() {
        let memo_contents = vec![
            read_test_file("src/test_files/output_parser/case6/memo1.txt"),
            read_test_file("src/test_files/output_parser/case6/memo2.txt"),
            read_test_file("src/test_files/output_parser/case6/memo3.txt"),
        ];
        let student_contents = vec![
            read_test_file("src/test_files/output_parser/case6/student1.txt"),
            read_test_file("src/test_files/output_parser/case6/student2.txt"),
            read_test_file("src/test_files/output_parser/case6/student3.txt"),
        ];
        let parser = OutputParser;
        let result = parser.parse(
            (&memo_contents, &student_contents, vec![3, 3, 3]),
            ExecutionConfig::default_config(),
        );

        assert!(result.is_ok());
        let submission = result.unwrap();
        assert_eq!(submission.tasks.len(), 3);

        for (i, task) in submission.tasks.iter().enumerate() {
            assert_eq!(task.task_id, format!("Task{}", i + 1));
            assert_eq!(task.memo_output.subtasks.len(), 3);
            assert_eq!(task.memo_output.subtasks[0].name, "Sub1");
            assert_eq!(task.memo_output.subtasks[0].lines.len(), 2);
            assert_eq!(task.memo_output.subtasks[1].name, "Sub2");
            assert_eq!(task.memo_output.subtasks[1].lines.len(), 2);

            assert_eq!(task.student_output.subtasks.len(), 3);
            assert_eq!(task.student_output.subtasks[0].name, "Sub1");
            assert_eq!(task.student_output.subtasks[0].lines.len(), 2);
            assert_eq!(task.student_output.subtasks[1].name, "Sub2");
            assert_eq!(task.student_output.subtasks[1].lines.len(), 2);
        }
    }
}
