//! Manual feedback strategy: allows instructors to specify custom feedback for each task.
//!
//! This is a stub implementation. In a real system, this would be populated from instructor input.

use crate::error::MarkerError;
use crate::traits::feedback::{Feedback, FeedbackEntry};
use crate::types::TaskResult;
use std::pin::Pin;

pub struct ManualFeedback;

impl Feedback for ManualFeedback {
    fn assemble_feedback<'a>(
        &'a self,
        _results: &[TaskResult],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<FeedbackEntry>, MarkerError>> + Send + 'a>> {
        Box::pin(async move {
            // TODO: Implement manual feedback assembly
            Err(MarkerError::InputMismatch(
                "Manual feedback not implemented".into(),
            ))
        })
    }
}
