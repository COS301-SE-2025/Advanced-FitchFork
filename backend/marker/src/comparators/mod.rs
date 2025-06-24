//! # Comparators
//!
//! This module provides a collection of comparators for evaluating student submissions.
//! Each comparator implements a specific logic for comparing the output of a student's code
//! with a predefined memo or expected output.
//!
//! All comparators in this module adhere to the `Comparator` trait, which defines a
//! common interface for comparison operations. This allows for flexible and interchangeable
//! comparison strategies within the marking system.
//!
//! The available comparators are:
//! - [`percentage_comparator`]: Compares two strings and calculates a similarity percentage.
//! - [`exact_comparator`]: Compares two strings and ensures that they match exactly.
//! - [`regex_comparator`]: Uses regular expressions to match patterns in the student's output.

pub mod percentage_comparator;
pub mod exact_comparator;
pub mod regex_comparator;