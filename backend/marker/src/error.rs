//! Marker Error Types
//!
//! This module defines the [`MarkerError`] enum, which encapsulates all error types that can occur during the parsing, validation, and loading of report and input files in the marker system.
//! Each variant provides a descriptive error message for robust error handling and debugging.
//!
//! # Usage
//!
//! Use [`MarkerError`] as the error type in functions that may fail due to input, parsing, or I/O issues. Each variant is tailored to a specific error scenario encountered in the marker pipeline.
//!
//! # Example
//!
//! ```rust
//! use marker::error::MarkerError;
//!
//! fn parse_input(data: &str) -> Result<(), MarkerError> {
//!     if data.is_empty() {
//!         return Err(MarkerError::MissingField("input data".to_string()));
//!     }
//!     Ok(())
//! }
//! ```

/// Represents all error types that can occur in the marker system.
#[derive(Debug)]
pub enum MarkerError {
    /// Input arrays do not match in length or structure.
    InputMismatch(String),
    /// JSON is malformed or does not match expected schema.
    InvalidJson(String),
    /// A required field is missing from input.
    MissingField(String),
    /// I/O error (file not found, unreadable, etc.).
    IoError(String),
    /// Mark allocation weights do not match expected totals.
    WeightMismatch(String),
    /// A required task ID is missing in coverage or complexity report.
    MissingTaskId(String),
    /// Error parsing coverage report (schema or content error).
    ParseCoverageError(String),
    /// Error parsing mark allocator (schema or content error).
    ParseAllocatorError(String),
    /// Error parsing complexity report (schema or content error).
    ParseComplexityError(String),
} 