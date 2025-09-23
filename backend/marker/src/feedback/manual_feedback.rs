//! Manual feedback strategy: allows instructors to specify custom feedback for each task.
//!
//! This strategy uses manual feedback stored in TaskResult to provide feedback
//! based on the student's performance and manual feedback configuration.

use crate::error::MarkerError;
use crate::traits::feedback::{Feedback, FeedbackEntry};
use crate::types::TaskResult;
use std::future::Future;
use std::pin::Pin;

pub struct ManualFeedback;

impl Feedback for ManualFeedback {
    fn assemble_feedback<'a>(
        &'a self,
        results: &'a [TaskResult],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<FeedbackEntry>, MarkerError>> + Send + 'a>> {
        Box::pin(async move {
            let mut feedback_entries = Vec::new();

            for result in results {
                let percentage = if result.possible > 0 {
                    (result.awarded as f64 / result.possible as f64) * 100.0
                } else {
                    0.0
                };

                let feedback_message = if percentage >= 100.0 {
                    "All patterns matched".to_string()
                } else if let Some(ref manual_feedback) = result.manual_feedback {
                    manual_feedback.clone()
                } else {
                    format!(
                        "Score: {:.1}% - Some patterns were not matched correctly",
                        percentage
                    )
                };

                feedback_entries.push(FeedbackEntry {
                    task: result.name.clone(),
                    message: feedback_message,
                });
            }

            Ok(feedback_entries)
        })
    }
}
