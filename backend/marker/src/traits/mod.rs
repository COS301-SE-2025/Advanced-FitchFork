//!
//! Traits Module
//!
//! This module contains core traits used throughout the marker system for extensibility and abstraction.
//!
//! - [`comparator`]: Defines traits for comparing outputs and reports.
//! - [`report_parser`]: Defines the generic trait for parsing JSON reports into Rust types.
//!
//! Implement these traits to extend or customize the marker's behavior for new report types or comparison strategies.

pub mod comparator; 
pub mod report_parser; 