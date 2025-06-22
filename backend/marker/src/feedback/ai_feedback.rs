//! AI feedback strategy: generates feedback using a Large Language Model (LLM).
//!
//! This is a stub implementation. In a real system, this would call an LLM service to generate feedback.

use crate::error::MarkerError;
use crate::traits::feedback::{Feedback, FeedbackEntry};
use crate::types::TaskResult;

pub struct AiFeedback;

impl Feedback for AiFeedback {
    fn assemble_feedback(&self, _results: &[TaskResult]) -> Result<Vec<FeedbackEntry>, MarkerError> {
        Err(MarkerError::InputMismatch("AI feedback not implemented".into()))
    }
}
