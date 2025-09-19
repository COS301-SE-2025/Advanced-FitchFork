//! Allocator Report Parser
//!
//! This module provides the [`JsonAllocatorParser`] for parsing allocator report JSON files into a strongly-typed Rust schema [`AllocatorSchema`].
//! The expected JSON format is validated strictly, with detailed error reporting for schema mismatches.
//!
//! # JSON Schema
//!
//! The expected JSON structure is as follows:
//!
//! ```json
//! {
//!   "generated_at": "<timestamp>",
//!   "total_value": <u32>,
//!   "tasks": [
//!     {
//!       "task1": {
//!         "name": "<task name>",
//!         "value": <u32>,
//!         "code_coverage": <boolean>,
//!         "subsections": [
//!           { "name": "<subsection name>", "value": <u32> },
//!           ...
//!         ]
//!       }
//!     },
//!     ...
//!   ]
//! }
//! ```
//!
//! - `generated_at` must be a string.
//! - `total_value` must be a number representing the sum of all task values.
//! - `tasks` is an array of objects, each with a single key (`task1`, `task2`, ...), whose value is an object with `name`, `value`, `code_coverage`, and `subsections` fields.
//! - Each subsection must have a `name` (string) and `value` (u32).
//! - The sum of subsection values must not exceed the parent task's value.
//!
//! # Error Handling
//!
//! Parsing errors are reported as [`MarkerError::ParseAllocatorError`] with descriptive messages.
//!
//! # Tests
//!
//! This module includes comprehensive tests for valid and invalid report files, covering edge cases and error reporting.

use crate::types::{AllocatorSchema, Subsection, TaskEntry};

/// Parser for allocator reports in JSON format.
///
/// Implements [`ReportParser<AllocatorSchema>`] for parsing and validating the schema.
pub struct JsonAllocatorParser;

use crate::error::MarkerError;
use crate::traits::parser::Parser;
use serde_json::Value;
use util::execution_config::ExecutionConfig;

impl<'a> Parser<&'a Value, AllocatorSchema> for JsonAllocatorParser {
    /// Parses a JSON value into an [`AllocatorSchema`].
    ///
    /// # Errors
    ///
    /// Returns [`MarkerError::ParseAllocatorError`] if the JSON does not conform to the expected schema.
    fn parse(
        &self,
        raw: &'a Value,
        _config: ExecutionConfig,
    ) -> Result<AllocatorSchema, MarkerError> {
        let obj = raw.as_object().ok_or_else(|| {
            MarkerError::ParseAllocatorError(
                "Top-level JSON must be an object to check 'generated_at' and 'tasks'".to_string(),
            )
        })?;

        match obj.get("generated_at") {
            Some(Value::String(_)) => {}
            Some(_) => {
                return Err(MarkerError::ParseAllocatorError(
                    "'generated_at' must be a string".to_string(),
                ));
            }
            None => {
                return Err(MarkerError::ParseAllocatorError(
                    "Missing required 'generated_at' field".to_string(),
                ));
            }
        }

        let arr = obj.get("tasks").and_then(|v| v.as_array()).ok_or_else(|| {
            MarkerError::ParseAllocatorError(
                "Top-level JSON must have a 'tasks' array field".to_string(),
            )
        })?;

        let total_value = match obj.get("total_value") {
            Some(Value::Number(n)) if n.is_u64() => n.as_u64().unwrap() as i64,
            Some(_) => {
                return Err(MarkerError::ParseAllocatorError(
                    "'total_value' must be a number".to_string(),
                ));
            }
            None => {
                return Err(MarkerError::ParseAllocatorError(
                    "Missing required 'total_value' field".to_string(),
                ));
            }
        };

        let mut tasks = Vec::with_capacity(arr.len());
        for (i, task_obj) in arr.iter().enumerate() {
            let obj = task_obj.as_object().ok_or_else(|| {
                MarkerError::ParseAllocatorError(format!(
                    "Task entry at index {} is not an object",
                    i
                ))
            })?;
            if obj.len() != 1 {
                return Err(MarkerError::ParseAllocatorError(format!(
                    "Task entry at index {} must have exactly one key (task_id)",
                    i
                )));
            }

            let (task_id, task_val) = obj.iter().next().unwrap();
            let expected_task_id = format!("task{}", i + 1);
            if task_id != &expected_task_id {
                return Err(MarkerError::ParseAllocatorError(format!(
                    "Task entry at index {} has invalid key '{}', expected '{}'",
                    i, task_id, expected_task_id
                )));
            }

            let task_map = task_val.as_object().ok_or_else(|| {
                MarkerError::ParseAllocatorError(format!(
                    "Task '{}' value is not an object",
                    task_id
                ))
            })?;

            let name = match task_map.get("name") {
                Some(Value::String(s)) => s.clone(),
                _ => {
                    return Err(MarkerError::ParseAllocatorError(format!(
                        "Task '{}' missing or invalid 'name' field",
                        task_id
                    )));
                }
            };

            let value = match task_map.get("value") {
                Some(Value::Number(n)) if n.is_u64() => n.as_u64().unwrap() as i64,
                _ => {
                    return Err(MarkerError::ParseAllocatorError(format!(
                        "Task '{}' missing or invalid 'value' field (must be u32)",
                        task_id
                    )));
                }
            };

            let subsections_val = task_map.get("subsections").ok_or_else(|| {
                MarkerError::ParseAllocatorError(format!(
                    "Task '{}' missing 'subsections' field",
                    task_id
                ))
            })?;

            let subsections_arr = subsections_val.as_array().ok_or_else(|| {
                MarkerError::ParseAllocatorError(format!(
                    "Task '{}' 'subsections' is not an array",
                    task_id
                ))
            })?;

            let mut subsections = Vec::with_capacity(subsections_arr.len());
            let mut subsections_sum: i64 = 0;
            for (j, sub_val) in subsections_arr.iter().enumerate() {
                let sub_obj = sub_val.as_object().ok_or_else(|| {
                    MarkerError::ParseAllocatorError(format!(
                        "Task '{}' subsection at index {} is not an object",
                        task_id, j
                    ))
                })?;

                let sub_name = match sub_obj.get("name") {
                    Some(Value::String(s)) => s.clone(),
                    _ => {
                        return Err(MarkerError::ParseAllocatorError(format!(
                            "Task '{}' subsection {} missing or invalid 'name' field",
                            task_id, j
                        )));
                    }
                };

                let sub_value = match sub_obj.get("value") {
                    Some(Value::Number(n)) if n.is_u64() => {
                        let v = n.as_u64().unwrap() as i64;
                        subsections_sum = subsections_sum.saturating_add(v);
                        v
                    }
                    _ => {
                        return Err(MarkerError::ParseAllocatorError(format!(
                            "Task '{}' subsection {} missing or invalid 'value' field (must be u32)",
                            task_id, j
                        )));
                    }
                };

                let feedback = sub_obj
                    .get("feedback")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let regex = sub_obj.get("regex").and_then(|v| v.as_array()).map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>()
                });

                subsections.push(Subsection {
                    name: sub_name,
                    value: sub_value as i64,
                    feedback,
                    regex,
                });
            }

            if subsections_sum > value {
                return Err(MarkerError::ParseAllocatorError(format!(
                    "Task '{}' sum of subsection values ({}) exceeds parent value ({})",
                    task_id, subsections_sum, value
                )));
            }

            let code_coverage = match task_map.get("code_coverage") {
                Some(Value::Bool(b)) => *b,
                Some(_) => {
                    return Err(MarkerError::ParseAllocatorError(format!(
                        "Task '{}' has invalid 'code_coverage' field (must be boolean)",
                        task_id
                    )));
                }
                None => false,
            };

            tasks.push(TaskEntry {
                id: task_id.clone(),
                name,
                value: value as i64,
                code_coverage,
                subsections,
            });
        }

        Ok(AllocatorSchema { tasks, total_value })
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the allocator parser.
    //! These tests cover valid and invalid report files, including edge cases and error reporting.
    use super::*;
    use serde_json::Value;
    use std::fs;
    use std::path::Path;

    /// Test parsing a valid report with a single task and subsections.
    #[test]
    fn test_parse_valid_single_task() {
        let path = Path::new("src/test_files/allocator_parser/allocator_report_1.json");
        let data = fs::read_to_string(path).expect("Failed to read test JSON file");
        let value: Value = serde_json::from_str(&data).expect("Failed to parse JSON");
        let parser = JsonAllocatorParser;
        let report = parser
            .parse(&value, ExecutionConfig::default_config())
            .expect("Should parse valid single task report");
        assert_eq!(report.tasks.len(), 1, "Should have one task");
        let task = &report.tasks[0];
        assert_eq!(task.id, "task1");
        assert_eq!(task.name, "Initialization");
        assert_eq!(task.value, 10);
        assert_eq!(task.subsections.len(), 2);
        assert_eq!(task.subsections[0].name, "Constructor");
        assert_eq!(task.subsections[0].value, 6);
        assert_eq!(task.subsections[0].feedback, None);
        assert_eq!(task.subsections[1].name, "Setup Globals");
        assert_eq!(task.subsections[1].value, 4);
        assert_eq!(task.subsections[1].feedback, None);
        assert_eq!(report.total_value, 10);
    }

    /// Test parsing a valid report with multiple tasks, including a task with no subsections.
    #[test]
    fn test_parse_valid_multiple_tasks() {
        let path = Path::new("src/test_files/allocator_parser/allocator_report_2.json");
        let data = fs::read_to_string(path).expect("Failed to read test JSON file");
        let value: Value = serde_json::from_str(&data).expect("Failed to parse JSON");
        let parser = JsonAllocatorParser;
        let report = parser
            .parse(&value, ExecutionConfig::default_config())
            .expect("Should parse valid multiple tasks report");
        assert_eq!(report.tasks.len(), 2, "Should have two tasks");
        let task1 = &report.tasks[0];
        assert_eq!(task1.id, "task1");
        assert_eq!(task1.name, "Parsing Stage");
        assert_eq!(task1.value, 8);
        assert_eq!(task1.subsections.len(), 2);
        let task2 = &report.tasks[1];
        assert_eq!(task2.id, "task2");
        assert_eq!(task2.name, "Computation");
        assert_eq!(task2.value, 12);
        assert_eq!(task2.subsections.len(), 0);
        assert_eq!(report.total_value, 20);
    }

    /// Test error handling for a task missing the 'name' field.
    #[test]
    fn test_parse_missing_name() {
        let path = Path::new("src/test_files/allocator_parser/allocator_report_3.json");
        let data = fs::read_to_string(path).expect("Failed to read test JSON file");
        let value: Value = serde_json::from_str(&data).expect("Failed to parse JSON");
        let parser = JsonAllocatorParser;
        let result = parser.parse(&value, ExecutionConfig::default_config());
        match result {
            Err(MarkerError::ParseAllocatorError(msg)) => {
                assert!(
                    msg.contains("name"),
                    "Error message should mention missing name, got: {}",
                    msg
                );
            }
            other => panic!(
                "Expected ParseAllocatorError for missing name, got: {:?}",
                other
            ),
        }
    }

    /// Test error handling when the sum of subsection values exceeds the parent task's value.
    #[test]
    fn test_parse_subsection_sum_exceeds_parent() {
        let path = Path::new("src/test_files/allocator_parser/allocator_report_4.json");
        let data = fs::read_to_string(path).expect("Failed to read test JSON file");
        let value: Value = serde_json::from_str(&data).expect("Failed to parse JSON");
        let parser = JsonAllocatorParser;
        let result = parser.parse(&value, ExecutionConfig::default_config());
        match result {
            Err(MarkerError::ParseAllocatorError(msg)) => {
                assert!(
                    msg.contains("sum of subsection values"),
                    "Error message should mention sum of subsection values, got: {}",
                    msg
                );
            }
            other => panic!(
                "Expected ParseAllocatorError for subsection sum > parent value, got: {:?}",
                other
            ),
        }
    }

    /// Test error handling for invalid subsection array and wrong value types.
    #[test]
    fn test_parse_subsections_not_array_and_value_wrong_type() {
        let path = Path::new("src/test_files/allocator_parser/allocator_report_5.json");
        let data = fs::read_to_string(path).expect("Failed to read test JSON file");
        let value: Value = serde_json::from_str(&data).expect("Failed to parse JSON");
        let parser = JsonAllocatorParser;
        let result = parser.parse(&value, ExecutionConfig::default_config());
        match result {
            Err(MarkerError::ParseAllocatorError(msg)) => {
                assert!(
                    msg.contains("invalid key")
                        || msg.contains("not an array")
                        || msg.contains("invalid 'value' field"),
                    "Error message should mention invalid key, not an array, or invalid value, got: {}",
                    msg
                );
            }
            other => panic!(
                "Expected ParseAllocatorError for invalid subsections or value type, got: {:?}",
                other
            ),
        }
    }

    /// Test error handling when the total_value field is missing.
    #[test]
    fn test_parse_missing_total_value() {
        let json_str = r#"
        {
          "generated_at": "2025-06-20T12:00:00+02:00",
          "tasks": [
            {
              "task1": {
                "name": "Test Task",
                "value": 10,
                "subsections": [
                  { "name": "Sub1", "value": 5 }
                ]
              }
            }
          ]
        }
        "#;
        let value: Value = serde_json::from_str(json_str).expect("Failed to parse JSON");
        let parser = JsonAllocatorParser;
        let result = parser.parse(&value, ExecutionConfig::default_config());
        match result {
            Err(MarkerError::ParseAllocatorError(msg)) => {
                assert!(
                    msg.contains("total_value"),
                    "Error message should mention missing total_value, got: {}",
                    msg
                );
            }
            other => panic!(
                "Expected ParseAllocatorError for missing total_value, got: {:?}",
                other
            ),
        }
    }

    /// Test error handling when the total_value field has the wrong type.
    #[test]
    fn test_parse_total_value_wrong_type() {
        let json_str = r#"
        {
          "generated_at": "2025-06-20T12:00:00+02:00",
          "total_value": "not_a_number",
          "tasks": [
            {
              "task1": {
                "name": "Test Task",
                "value": 10,
                "subsections": [
                  { "name": "Sub1", "value": 5 }
                ]
              }
            }
          ]
        }
        "#;
        let value: Value = serde_json::from_str(json_str).expect("Failed to parse JSON");
        let parser = JsonAllocatorParser;
        let result = parser.parse(&value, ExecutionConfig::default_config());
        match result {
            Err(MarkerError::ParseAllocatorError(msg)) => {
                assert!(
                    msg.contains("must be a number"),
                    "Error message should mention that total_value must be a number, got: {}",
                    msg
                );
            }
            other => panic!(
                "Expected ParseAllocatorError for wrong type total_value, got: {:?}",
                other
            ),
        }
    }
}
