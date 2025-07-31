//! Parser Trait
//!
//! This module defines the [`Parser`] trait, which provides a generic interface for parsing
//! various data formats into strongly-typed Rust structures. Implementations of this trait
//! are responsible for validating the input and converting it into the appropriate domain
//! model, returning detailed errors on failure.
//!
//! # Usage
//!
//! Implement this trait for any parser that converts an input type into a specific output type.
//!
//! # Example
//!
//! ```rust
//! use marker::error::MarkerError;
//! use marker::traits::parser::Parser;
//! use serde_json::Value;
//! use util::execution_config::ExecutionConfig;
//!
//! struct MyJsonParser;
//! struct MyReport;
//!
//! // Example for a JSON parser
//! impl<'a> Parser<&'a Value, MyReport> for MyJsonParser {
//!     fn parse(&self, raw: &'a Value, _config: ExecutionConfig) -> Result<MyReport, MarkerError> {
//!         // Dummy implementation
//!         Ok(MyReport)
//!     }
//! }
//! ```

use util::execution_config::ExecutionConfig;

use crate::error::MarkerError;

/// A generic trait for parsing data into a strongly-typed Rust structure.
///
/// Implementors should validate the input and return a domain-specific type
/// or a [`MarkerError`] on failure.
///
/// # Type Parameters
///
/// * `Input` - The input type to be parsed.
/// * `Output` - The output type produced by the parser.
pub trait Parser<Input, Output> {
    /// Parse an input value into the target type.
    ///
    /// # Arguments
    ///
    /// * `input` - The input value to parse.
    ///
    /// # Errors
    ///
    /// Returns a [`MarkerError`] if the input does not conform to the expected schema
    /// or cannot be parsed.
    fn parse(&self, input: Input, config: ExecutionConfig) -> Result<Output, MarkerError>;
}
