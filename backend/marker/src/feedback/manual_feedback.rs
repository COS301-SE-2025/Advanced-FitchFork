//! Manual feedback strategy: allows instructors to specify custom feedback for each task.
//!
//! This is a stub implementation. In a real system, this would be populated from instructor input.

use crate::error::MarkerError;
use crate::traits::feedback::{Feedback, FeedbackEntry};
use crate::types::TaskResult;
use async_trait::async_trait;

pub struct ManualFeedback;

#[async_trait]
impl Feedback for ManualFeedback {
    async fn assemble_feedback(
        &self,
        _results: &[TaskResult],
    ) -> Result<Vec<FeedbackEntry>, MarkerError> {
        // TODO: Implement manual feedback assembly
        Err(MarkerError::InputMismatch(
            "Manual feedback not implemented".into(),
        ))
    }
}
