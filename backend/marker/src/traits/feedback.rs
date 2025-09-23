//!
//! # Feedback Trait
//!
//! This module defines the [`Feedback`] trait and the [`FeedbackEntry`] struct, which are used to implement pluggable feedback strategies for the marker system.
//!
//! Each feedback strategy produces a list of feedback entries based on the results of marking tasks, allowing for flexible and extensible feedback generation (e.g., auto, manual, or AI-based feedback).
//!

use crate::error::MarkerError;
use crate::types::TaskResult;
use serde::Serialize;
use std::pin::Pin;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FeedbackEntry {
    pub task: String,
    pub message: String,
}

/// A trait for pluggable feedback strategies in the marker system.
///
/// Implement this trait to define how feedback is generated from a set of marking results.
/// Each strategy can produce feedback in a different way (e.g., template-based, instructor-specified, or AI-generated).
///
/// # Arguments
/// - `results`: A slice of [`TaskResult`]s for the submission.
///
/// # Returns
/// - `Ok(Vec<FeedbackEntry>)`: An ordered list of feedback entries for the submission.
/// - `Err(MarkerError)`: If feedback generation fails.
///
pub trait Feedback {
    fn assemble_feedback<'a>(
        &'a self,
        results: &'a [TaskResult],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<FeedbackEntry>, MarkerError>> + Send + 'a>>;
}
