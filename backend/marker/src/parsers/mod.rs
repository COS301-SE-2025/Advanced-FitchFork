//! # Parsers
//!
//! This module is responsible for parsing various types of analysis reports.
//! Each sub-module within this module is dedicated to parsing a specific type of report,
//! such as complexity reports, coverage reports, or allocator reports.
//!
//! The parsers implemented in this module adhere to the `ReportParser` trait,
//! ensuring a consistent interface for parsing different report formats.
//
//! The available parsers are:
//! - [`allocator_parser`]: For parsing memory allocation reports.
//! - [`complexity_parser`]: For parsing code complexity reports.
//! - [`coverage_parser`]: For parsing code coverage reports.

pub mod allocator_parser;
pub mod complexity_parser;
pub mod coverage_parser;