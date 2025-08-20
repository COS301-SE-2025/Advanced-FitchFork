//! Complexity Report Parser
//!
//! This module provides the [`JsonComplexityParser`] for parsing resource complexity report JSON files into a strongly-typed Rust schema [`ComplexityReport`].
//! The parser validates the expected JSON format and provides detailed error messages for schema mismatches.
//!
//! # JSON Schema
//!
//! The expected JSON structure is as follows:
//!
//! ```json
//! {
//!   "generated_at": "<timestamp>",
//!   "resource_metrics": {
//!     "user_time_s": <f64>,
//!     "system_time_s": <f64>,
//!     "wall_time_s": <f64>,
//!     "max_rss_kb": <u64>
//!     // optional: "average_rss_kb", "io_read_bytes", "io_write_bytes"
//!   }
//! }
//! ```
//!
//! - `generated_at` must be a string.
//! - `resource_metrics` is an object with required timing and memory fields.
//! - Optional fields may be present but are ignored by this parser.
//!
//! # Error Handling
//!
//! Parsing errors are reported as [`MarkerError::ParseComplexityError`] with descriptive messages.
//!
//! # Tests
//!
//! This module includes comprehensive tests for valid and invalid report files, covering edge cases and error reporting.

/// The top-level schema for a complexity report, containing resource usage metrics.
#[derive(Debug)]
pub struct ComplexityReport {
    /// User CPU time in seconds.
    pub user_time_s: f64,
    /// System CPU time in seconds.
    pub system_time_s: f64,
    /// Wall clock time in seconds.
    pub wall_time_s: f64,
    /// Maximum resident set size (memory usage) in kilobytes.
    pub max_rss_kb: u64,
    /// Timestamp when the report was generated.
    pub generated_at: String,
    // optional: average_rss_kb, io_read_bytes, io_write_bytes
}

/// Parser for complexity reports in JSON format.
///
/// Implements [`ReportParser<ComplexityReport>`] for parsing and validating the schema.
pub struct JsonComplexityParser;

use crate::error::MarkerError;
use crate::traits::parser::Parser;
use serde_json::Value;
use util::execution_config::ExecutionConfig;

impl<'a> Parser<&'a Value, ComplexityReport> for JsonComplexityParser {
    /// Parses a JSON value into a [`ComplexityReport`].
    ///
    /// # Errors
    ///
    /// Returns [`MarkerError::ParseComplexityError`] if the JSON does not conform to the expected schema.
    fn parse(
        &self,
        raw: &'a Value,
        _config: ExecutionConfig,
    ) -> Result<ComplexityReport, MarkerError> {
        let obj = raw.as_object().ok_or_else(|| {
            MarkerError::ParseComplexityError("Top-level JSON must be an object".to_string())
        })?;

        let metrics = obj
            .get("resource_metrics")
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                MarkerError::ParseComplexityError(
                    "Missing or invalid 'resource_metrics' object".to_string(),
                )
            })?;

        let user_time_s = match metrics.get("user_time_s") {
            Some(Value::Number(n)) if n.is_f64() => n.as_f64().unwrap(),
            Some(Value::Number(n)) if n.is_u64() => n.as_u64().unwrap() as f64,
            Some(Value::Number(n)) if n.is_i64() => n.as_i64().unwrap() as f64,
            _ => {
                return Err(MarkerError::ParseComplexityError(
                    "'user_time_s' missing or not a number".to_string(),
                ));
            }
        };

        let system_time_s = match metrics.get("system_time_s") {
            Some(Value::Number(n)) if n.is_f64() => n.as_f64().unwrap(),
            Some(Value::Number(n)) if n.is_u64() => n.as_u64().unwrap() as f64,
            Some(Value::Number(n)) if n.is_i64() => n.as_i64().unwrap() as f64,
            _ => {
                return Err(MarkerError::ParseComplexityError(
                    "'system_time_s' missing or not a number".to_string(),
                ));
            }
        };

        let wall_time_s = match metrics.get("wall_time_s") {
            Some(Value::Number(n)) if n.is_f64() => n.as_f64().unwrap(),
            Some(Value::Number(n)) if n.is_u64() => n.as_u64().unwrap() as f64,
            Some(Value::Number(n)) if n.is_i64() => n.as_i64().unwrap() as f64,
            _ => {
                return Err(MarkerError::ParseComplexityError(
                    "'wall_time_s' missing or not a number".to_string(),
                ));
            }
        };

        let max_rss_kb = match metrics.get("max_rss_kb") {
            Some(Value::Number(n)) if n.is_u64() => n.as_u64().unwrap(),
            _ => {
                return Err(MarkerError::ParseComplexityError(
                    "'max_rss_kb' missing or not u64".to_string(),
                ));
            }
        };

        let generated_at = obj
            .get("generated_at")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                MarkerError::ParseComplexityError(
                    "'generated_at' missing or not a string".to_string(),
                )
            })?
            .to_string();

        Ok(ComplexityReport {
            user_time_s,
            system_time_s,
            wall_time_s,
            max_rss_kb,
            generated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the complexity parser.
    //! These tests cover valid and invalid report files, including edge cases and error reporting.
    use super::*;
    use serde_json::Value;
    use std::fs;
    use std::path::Path;

    /// Helper for approximate float equality.
    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-8
    }

    /// Test parsing a valid report with fast and light resource usage.
    #[test]
    fn test_parse_valid_fast_light_report() {
        // Load the JSON file
        let path = Path::new("src/test_files/complexity_parser/complexity_report_1.json");
        let data = fs::read_to_string(path).expect("Failed to read test JSON file");
        let value: Value = serde_json::from_str(&data).expect("Failed to parse JSON");

        // Parse using the parser
        let parser = JsonComplexityParser;
        let report = parser
            .parse(&value, ExecutionConfig::default_config())
            .expect("Should parse valid report");

        // Assert all fields
        assert!(approx_eq(report.user_time_s, 0.12), "user_time_s");
        assert!(approx_eq(report.system_time_s, 0.03), "system_time_s");
        assert!(approx_eq(report.wall_time_s, 0.20), "wall_time_s");
        assert_eq!(report.max_rss_kb, 51200, "max_rss_kb");
        assert_eq!(report.generated_at, "2025-06-20T12:00:00+02:00");
    }

    /// Test parsing a valid report with integer time values.
    #[test]
    fn test_parse_valid_integer_times_report() {
        // Load the JSON file
        let path =
            std::path::Path::new("src/test_files/complexity_parser/complexity_report_2.json");
        let data = std::fs::read_to_string(path).expect("Failed to read test JSON file");
        let value: serde_json::Value = serde_json::from_str(&data).expect("Failed to parse JSON");

        // Parse using the parser
        let parser = JsonComplexityParser;
        let report = parser
            .parse(&value, ExecutionConfig::default_config())
            .expect("Should parse valid report");

        // Assert all fields
        assert!(approx_eq(report.user_time_s, 1.0), "user_time_s");
        assert!(approx_eq(report.system_time_s, 0.0), "system_time_s");
        assert!(approx_eq(report.wall_time_s, 2.0), "wall_time_s");
        assert_eq!(report.max_rss_kb, 1048576, "max_rss_kb");
        assert_eq!(report.generated_at, "2025-06-20T12:01:00+02:00");
    }

    /// Test error handling for missing required fields.
    #[test]
    fn test_parse_missing_fields_report() {
        // Load the JSON file
        let path =
            std::path::Path::new("src/test_files/complexity_parser/complexity_report_3.json");
        let data = std::fs::read_to_string(path).expect("Failed to read test JSON file");
        let value: serde_json::Value = serde_json::from_str(&data).expect("Failed to parse JSON");

        // Parse using the parser
        let parser = JsonComplexityParser;
        let result = parser.parse(&value, ExecutionConfig::default_config());

        // Should error due to missing user_time_s
        match result {
            Err(crate::error::MarkerError::ParseComplexityError(msg)) => {
                assert!(
                    msg.contains("user_time_s"),
                    "Error message should mention missing user_time_s, got: {}",
                    msg
                );
            }
            other => panic!(
                "Expected ParseComplexityError for missing user_time_s, got: {:?}",
                other
            ),
        }
    }

    /// Test error handling for wrong type in max_rss_kb.
    #[test]
    fn test_parse_wrong_type_report() {
        // Load the JSON file
        let path =
            std::path::Path::new("src/test_files/complexity_parser/complexity_report_4.json");
        let data = std::fs::read_to_string(path).expect("Failed to read test JSON file");
        let value: serde_json::Value = serde_json::from_str(&data).expect("Failed to parse JSON");

        // Parse using the parser
        let parser = JsonComplexityParser;
        let result = parser.parse(&value, ExecutionConfig::default_config());

        // Should error due to max_rss_kb being a string instead of u64
        match result {
            Err(crate::error::MarkerError::ParseComplexityError(msg)) => {
                assert!(
                    msg.contains("max_rss_kb"),
                    "Error message should mention max_rss_kb, got: {}",
                    msg
                );
            }
            other => panic!(
                "Expected ParseComplexityError for wrong type max_rss_kb, got: {:?}",
                other
            ),
        }
    }

    /// Test parsing a report with extra/optional fields present.
    #[test]
    fn test_parse_extended_metrics_report() {
        // Load the JSON file
        let path =
            std::path::Path::new("src/test_files/complexity_parser/complexity_report_5.json");
        let data = std::fs::read_to_string(path).expect("Failed to read test JSON file");
        let value: serde_json::Value = serde_json::from_str(&data).expect("Failed to parse JSON");

        // Parse using the parser
        let parser = JsonComplexityParser;
        let report = parser
            .parse(&value, ExecutionConfig::default_config())
            .expect("Should parse valid report with extra fields");

        // Assert all required fields
        assert!(approx_eq(report.user_time_s, 0.30), "user_time_s");
        assert!(approx_eq(report.system_time_s, 0.07), "system_time_s");
        assert!(approx_eq(report.wall_time_s, 0.45), "wall_time_s");
        assert_eq!(report.max_rss_kb, 76800, "max_rss_kb");
        assert_eq!(report.generated_at, "2025-06-20T12:04:00+02:00");
    }
}
