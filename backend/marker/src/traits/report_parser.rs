//! Report Parser Trait
//!
//! This module defines the [`ReportParser`] trait, which provides a generic interface for parsing JSON report data into strongly-typed Rust structures.
//! Implementations of this trait are responsible for validating the input JSON and converting it into the appropriate domain model, returning detailed errors on failure.
//!
//! # Usage
//!
//! Implement this trait for any parser that converts a `serde_json::Value` into a specific report type, such as coverage, complexity, or allocator reports.
//!
//! # Example
//!
//! ```rust
//! use serde_json::Value;
//! use marker::error::MarkerError;
//! use marker::traits::report_parser::ReportParser;
//!
//! struct MyReportParser;
//! struct MyReport;
//!
//! impl ReportParser<MyReport> for MyReportParser {
//!     fn parse(&self, raw: &Value) -> Result<MyReport, MarkerError> {
//!         Ok(MyReport)
//!     }
//! }
//! ```

use crate::error::MarkerError;
use serde_json::Value;

/// A trait for parsing JSON report data into a strongly-typed Rust structure.
///
/// Implementors should validate the input JSON and return a domain-specific type or a [`MarkerError`] on failure.
///
/// # Type Parameters
///
/// * `T` - The output type produced by the parser.
pub trait ReportParser<T> {
    /// Parse a JSON value into the target type.
    ///
    /// # Arguments
    ///
    /// * `raw` - The input JSON value to parse.
    ///
    /// # Errors
    ///
    /// Returns a [`MarkerError`] if the input does not conform to the expected schema or cannot be parsed.
    fn parse(&self, raw: &Value) -> Result<T, MarkerError>;
}