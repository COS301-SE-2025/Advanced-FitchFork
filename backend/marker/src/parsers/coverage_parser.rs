//! Coverage Report Parser
//!
//! This module provides the [`JsonCoverageParser`] for parsing code coverage report JSON files into a strongly-typed Rust schema [`CoverageReport`].
//! The parser validates the expected JSON format and provides detailed error messages for schema mismatches.
//!
//! # JSON Schema
//!
//! The expected JSON structure is as follows:
//!
//! ```json
//! {
//!   "generated_at": "<timestamp>",
//!   "summary": {
//!     "total_files": <u64>,
//!     "total_lines": <u64>,
//!     "covered_lines": <u64>,
//!     "coverage_percent": <f64>
//!   },
//!   "files": [
//!     {
//!       "path": "<file path>",
//!       "total_lines": <u64>,
//!       "covered_lines": <u64>,
//!       "coverage_percent": <f64>
//!     },
//!     ...
//!   ]
//! }
//! ```
//!
//! - `generated_at` must be a string.
//! - `summary` is an object with total/covered lines and percent fields.
//! - `files` is an array of file coverage objects, each with path, line counts, and percent.
//!
//! # Error Handling
//!
//! Parsing errors are reported as [`MarkerError::ParseCoverageError`] with descriptive messages.
//!
//! # Tests
//!
//! This module includes comprehensive tests for valid and invalid report files, covering edge cases and error reporting.

/// The top-level schema for a coverage report, containing summary and per-file coverage.
#[derive(Debug)]
pub struct CoverageReport {
    /// Total number of files in the report.
    pub total_files: u64,
    /// Total number of lines across all files.
    pub total_lines: u64,
    /// Total number of covered lines across all files.
    pub covered_lines: u64,
    /// Overall coverage percentage (0.0 - 100.0).
    pub coverage_percent: f64,
    /// Per-file coverage details.
    pub files: Vec<FileCoverage>,
}

/// Represents coverage information for a single file.
#[derive(Debug)]
pub struct FileCoverage {
    /// The file path.
    pub path: String,
    /// Total number of lines in the file.
    pub total_lines: u64,
    /// Number of covered lines in the file.
    pub covered_lines: u64,
    /// Coverage percentage for the file (0.0 - 100.0).
    pub coverage_percent: f64,
    // per-line details omitted
}

/// Parser for coverage reports in JSON format.
///
/// Implements [`ReportParser<CoverageReport>`] for parsing and validating the schema.
pub struct JsonCoverageParser;

use crate::error::MarkerError;
use crate::traits::parser::Parser;
use serde_json::Value;
use util::execution_config::ExecutionConfig;

impl<'a> Parser<&'a Value, CoverageReport> for JsonCoverageParser {
    /// Parses a JSON value into a [`CoverageReport`].
    ///
    /// # Errors
    ///
    /// Returns [`MarkerError::ParseCoverageError`] if the JSON does not conform to the expected schema.
    fn parse(
        &self,
        raw: &'a Value,
        _config: ExecutionConfig,
    ) -> Result<CoverageReport, MarkerError> {
        let obj = raw.as_object().ok_or_else(|| {
            MarkerError::ParseCoverageError("Top-level JSON must be an object".to_string())
        })?;

        let summary = obj
            .get("summary")
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                MarkerError::ParseCoverageError("Missing or invalid 'summary' object".to_string())
            })?;

        let _ts = obj
            .get("generated_at")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                MarkerError::ParseCoverageError("Missing or invalid 'generated_at'".into())
            })?;

        let total_files = match summary.get("total_files") {
            Some(Value::Number(n)) if n.is_u64() => n.as_u64().unwrap(),
            _ => {
                return Err(MarkerError::ParseCoverageError(
                    "'summary.total_files' missing or not u64".to_string(),
                ));
            }
        };

        let total_lines = match summary.get("total_lines") {
            Some(Value::Number(n)) if n.is_u64() => n.as_u64().unwrap(),
            _ => {
                return Err(MarkerError::ParseCoverageError(
                    "'summary.total_lines' missing or not u64".to_string(),
                ));
            }
        };

        let covered_lines = match summary.get("covered_lines") {
            Some(Value::Number(n)) if n.is_u64() => n.as_u64().unwrap(),
            _ => {
                return Err(MarkerError::ParseCoverageError(
                    "'summary.covered_lines' missing or not u64".to_string(),
                ));
            }
        };

        let coverage_percent = match summary.get("coverage_percent") {
            Some(Value::Number(n)) if n.is_f64() => n.as_f64().unwrap(),
            Some(Value::Number(n)) if n.is_u64() => n.as_u64().unwrap() as f64,
            Some(Value::Number(n)) if n.is_i64() => n.as_i64().unwrap() as f64,
            _ => {
                return Err(MarkerError::ParseCoverageError(
                    "'summary.coverage_percent' missing or not f64".to_string(),
                ));
            }
        };

        let files_val = obj
            .get("files")
            .ok_or_else(|| MarkerError::ParseCoverageError("Missing 'files' array".to_string()))?;

        let files_arr = files_val.as_array().ok_or_else(|| {
            MarkerError::ParseCoverageError("'files' is not an array".to_string())
        })?;

        let mut files = Vec::with_capacity(files_arr.len());
        for (i, file_val) in files_arr.iter().enumerate() {
            let file_obj = file_val.as_object().ok_or_else(|| {
                MarkerError::ParseCoverageError(format!(
                    "File entry at index {} is not an object",
                    i
                ))
            })?;

            let path = match file_obj.get("path") {
                Some(Value::String(s)) => s.clone(),
                _ => {
                    return Err(MarkerError::ParseCoverageError(format!(
                        "File {} missing or invalid 'path' field",
                        i
                    )));
                }
            };

            let total_lines = match file_obj.get("total_lines") {
                Some(Value::Number(n)) if n.is_u64() => n.as_u64().unwrap(),
                _ => {
                    return Err(MarkerError::ParseCoverageError(format!(
                        "File {} missing or invalid 'total_lines' field",
                        i
                    )));
                }
            };

            let covered_lines = match file_obj.get("covered_lines") {
                Some(Value::Number(n)) if n.is_u64() => n.as_u64().unwrap(),
                _ => {
                    return Err(MarkerError::ParseCoverageError(format!(
                        "File {} missing or invalid 'covered_lines' field",
                        i
                    )));
                }
            };

            let coverage_percent = match file_obj.get("coverage_percent") {
                Some(Value::Number(n)) if n.is_f64() => n.as_f64().unwrap(),
                Some(Value::Number(n)) if n.is_u64() => n.as_u64().unwrap() as f64,
                Some(Value::Number(n)) if n.is_i64() => n.as_i64().unwrap() as f64,
                _ => {
                    return Err(MarkerError::ParseCoverageError(format!(
                        "File {} missing or invalid 'coverage_percent' field",
                        i
                    )));
                }
            };

            files.push(FileCoverage {
                path,
                total_lines,
                covered_lines,
                coverage_percent,
            });
        }

        Ok(CoverageReport {
            total_files,
            total_lines,
            covered_lines,
            coverage_percent,
            files,
        })
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the coverage parser.
    //! These tests cover valid and invalid report files, including edge cases and error reporting.
    use super::*;
    use serde_json::Value;
    use std::fs;
    use std::path::Path;

    /// Helper for approximate float equality.
    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-8
    }

    /// Test parsing a valid report with a single file.
    #[test]
    fn test_parse_valid_single_file_report() {
        // Load the JSON file
        let path = Path::new("src/test_files/coverage_parser/coverage_report_1.json");
        let data = fs::read_to_string(path).expect("Failed to read test JSON file");
        let value: Value = serde_json::from_str(&data).expect("Failed to parse JSON");

        // Parse using the parser
        let parser = JsonCoverageParser;
        let report = parser
            .parse(&value, ExecutionConfig::default_config())
            .expect("Should parse valid report");

        // Assert all fields
        assert_eq!(report.total_files, 1, "total_files");
        assert_eq!(report.total_lines, 100, "total_lines");
        assert_eq!(report.covered_lines, 90, "covered_lines");
        assert!(approx_eq(report.coverage_percent, 90.0), "coverage_percent");
        assert_eq!(report.files.len(), 1, "files.len");
        let file = &report.files[0];
        assert_eq!(file.path, "src/lib.rs", "file.path");
        assert_eq!(file.total_lines, 100, "file.total_lines");
        assert_eq!(file.covered_lines, 90, "file.covered_lines");
        assert!(
            approx_eq(file.coverage_percent, 90.0),
            "file.coverage_percent"
        );
    }

    /// Test parsing a valid report with multiple files and extra fields.
    #[test]
    fn test_parse_valid_multiple_files_extra_fields_report() {
        // Load the JSON file
        let path = std::path::Path::new("src/test_files/coverage_parser/coverage_report_2.json");
        let data = std::fs::read_to_string(path).expect("Failed to read test JSON file");
        let value: serde_json::Value = serde_json::from_str(&data).expect("Failed to parse JSON");

        // Parse using the parser
        let parser = JsonCoverageParser;
        let report = parser
            .parse(&value, ExecutionConfig::default_config())
            .expect("Should parse valid report");

        // Assert summary fields
        assert_eq!(report.total_files, 2, "total_files");
        assert_eq!(report.total_lines, 300, "total_lines");
        assert_eq!(report.covered_lines, 210, "covered_lines");
        assert!(approx_eq(report.coverage_percent, 70.0), "coverage_percent");
        assert_eq!(report.files.len(), 2, "files.len");

        // Assert first file
        let file1 = &report.files[0];
        assert_eq!(file1.path, "src/parser.rs", "file1.path");
        assert_eq!(file1.total_lines, 150, "file1.total_lines");
        assert_eq!(file1.covered_lines, 120, "file1.covered_lines");
        assert!(
            approx_eq(file1.coverage_percent, 80.0),
            "file1.coverage_percent"
        );

        // Assert second file
        let file2 = &report.files[1];
        assert_eq!(file2.path, "src/processor.rs", "file2.path");
        assert_eq!(file2.total_lines, 150, "file2.total_lines");
        assert_eq!(file2.covered_lines, 90, "file2.covered_lines");
        assert!(
            approx_eq(file2.coverage_percent, 60.0),
            "file2.coverage_percent"
        );
    }

    /// Test error handling for missing summary object.
    #[test]
    fn test_parse_missing_summary_report() {
        // Load the JSON file
        let path = std::path::Path::new("src/test_files/coverage_parser/coverage_report_3.json");
        let data = std::fs::read_to_string(path).expect("Failed to read test JSON file");
        let value: serde_json::Value = serde_json::from_str(&data).expect("Failed to parse JSON");

        // Parse using the parser
        let parser = JsonCoverageParser;
        let result = parser.parse(&value, ExecutionConfig::default_config());

        // Should error due to missing summary
        match result {
            Err(crate::error::MarkerError::ParseCoverageError(msg)) => {
                assert!(
                    msg.contains("summary"),
                    "Error message should mention missing summary, got: {}",
                    msg
                );
            }
            other => panic!(
                "Expected ParseCoverageError for missing summary, got: {:?}",
                other
            ),
        }
    }

    /// Test error handling for wrong type in summary.total_lines.
    #[test]
    fn test_parse_summary_total_lines_string_report() {
        // Load the JSON file
        let path = std::path::Path::new("src/test_files/coverage_parser/coverage_report_4.json");
        let data = std::fs::read_to_string(path).expect("Failed to read test JSON file");
        let value: serde_json::Value = serde_json::from_str(&data).expect("Failed to parse JSON");

        // Parse using the parser
        let parser = JsonCoverageParser;
        let result = parser.parse(&value, ExecutionConfig::default_config());

        // Should error due to summary.total_lines being a string
        match result {
            Err(crate::error::MarkerError::ParseCoverageError(msg)) => {
                assert!(
                    msg.contains("total_lines"),
                    "Error message should mention total_lines, got: {}",
                    msg
                );
            }
            other => panic!(
                "Expected ParseCoverageError for wrong type total_lines, got: {:?}",
                other
            ),
        }
    }

    /// Test error handling for missing file path in a file entry.
    #[test]
    fn test_parse_file_entry_missing_path_report() {
        // Load the JSON file
        let path = std::path::Path::new("src/test_files/coverage_parser/coverage_report_5.json");
        let data = std::fs::read_to_string(path).expect("Failed to read test JSON file");
        let value: serde_json::Value = serde_json::from_str(&data).expect("Failed to parse JSON");

        // Parse using the parser
        let parser = JsonCoverageParser;
        let result = parser.parse(&value, ExecutionConfig::default_config());

        // Should error due to file entry missing 'path'
        match result {
            Err(crate::error::MarkerError::ParseCoverageError(msg)) => {
                assert!(
                    msg.contains("path"),
                    "Error message should mention missing path, got: {}",
                    msg
                );
            }
            other => panic!(
                "Expected ParseCoverageError for missing file path, got: {:?}",
                other
            ),
        }
    }
}
