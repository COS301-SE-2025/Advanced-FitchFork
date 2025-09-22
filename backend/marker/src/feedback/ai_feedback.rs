//! # AI Feedback Strategy
//!
//! This module provides an implementation of the [`Feedback`] trait that generates feedback for student submissions using a Large Language Model (LLM), specifically Google's Gemini API. The AI feedback strategy is designed to provide concise, constructive hints to students based on which patterns they missed in their code, without revealing the answer. If all patterns are matched, a positive message is returned instead.
//!
//! ## Overview
//!
//! - The [`AiFeedback`] struct implements the [`Feedback`] trait asynchronously.
//! - For each [`TaskResult`], it generates a prompt describing the missed patterns and sends it to the Gemini API.
//! - The API response is parsed and the resulting hint is returned as a [`FeedbackEntry`].
//! - If all patterns are matched, a default congratulatory message is returned.
//!
//! ## Environment
//!
//! - Requires the `GEMINI_API_KEY` environment variable to be set for authenticating with the Gemini API.
//!
//! ## Testing
//!
//! - Includes a test that mocks two tasks: one with missed patterns and one with all patterns matched, verifying the feedback generation logic.
//!
//! ## Note
//!
//! This is a stub implementation. In a production system, error handling, rate limiting, and prompt engineering should be more robust.

use crate::error::MarkerError;
use crate::traits::feedback::{Feedback, FeedbackEntry};
use crate::types::TaskResult;
use serde::{Deserialize, Serialize};
use serde_json;
use util::config;

/// AI feedback strategy: generates feedback using a Large Language Model (LLM).
///
/// This struct implements the [`Feedback`] trait and provides AI-generated feedback for student submissions.
pub struct AiFeedback;

/// Request body for the Gemini API.
#[derive(Serialize)]
struct GeminiRequest {
    /// The content to send to the LLM.
    contents: Vec<Content>,
    /// Optional generation configuration for the LLM.
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

/// Content wrapper for the Gemini API request.
#[derive(Serialize)]
struct Content {
    /// The parts of the message (e.g., prompt text).
    parts: Vec<Part>,
}

/// A single part of the content, typically a text prompt.
#[derive(Serialize)]
struct Part {
    /// The text content to send to the LLM.
    text: String,
}

/// Response from the Gemini API.
#[derive(Deserialize)]
struct GeminiResponse {
    /// List of candidate completions from the LLM.
    candidates: Vec<Candidate>,
}

/// A single candidate response from the Gemini API.
#[derive(Deserialize)]
struct Candidate {
    /// The content of the candidate response.
    content: ContentResponse,
}

/// Content of a candidate response.
#[derive(Deserialize)]
struct ContentResponse {
    /// The parts of the response (e.g., generated hint text).
    parts: Vec<PartResponse>,
}

/// A single part of the response content.
#[derive(Deserialize)]
struct PartResponse {
    /// The generated text from the LLM.
    text: String,
}

/// Optional configuration for the LLM generation process.
#[derive(Serialize)]
struct GenerationConfig {
    /// Configuration for the LLM's thinking process.
    thinking_config: ThinkingConfig,
}

/// Configuration for the LLM's thinking process.
#[derive(Serialize)]
struct ThinkingConfig {
    /// The thinking budget for the LLM (set to 0 to disable thinking for faster requests).
    thinking_budget: u32,
}

impl Feedback for AiFeedback {
    /// Assembles feedback for a list of [`TaskResult`]s using the Gemini LLM API.
    ///
    /// For each task:
    /// - If all patterns are matched, returns a congratulatory message.
    /// - If there are missed patterns, sends a prompt to the Gemini API to generate a hint.
    ///
    /// # Arguments
    ///
    /// * `results` - A slice of [`TaskResult`]s representing the outcome of student tasks.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of [`FeedbackEntry`]s or a [`MarkerError`].
    fn assemble_feedback<'a>(
        &'a self,
        results: &'a [TaskResult],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<FeedbackEntry>, MarkerError>> + Send + 'a>> {
        Box::pin(async move {
            dotenvy::dotenv().ok();

            let api_key = config::gemini_api_key();

            let client = reqwest::Client::new();
            let mut feedback_entries = Vec::new();

            for result in results {
                let message = if result.missed_patterns.is_empty() {
                    "All patterns matched".to_string()
                } else {
                    let prompt = format!(
                        "For a task named '{}', the student missed the following patterns:\n{}\nPlease provide a short and concise hint to the student without giving away the answer.",
                        result.name,
                        result.missed_patterns.join("\n")
                    );

                    let request_body = GeminiRequest {
                        contents: vec![Content {
                            parts: vec![Part { text: prompt }],
                        }],
                        generation_config: Some(GenerationConfig {
                            thinking_config: ThinkingConfig { thinking_budget: 0 },
                        }),
                    };

                    let response = client
                        .post(format!(
                            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
                            api_key
                        ))
                        .json(&request_body)
                        .send()
                        .await
                        .map_err(|e| MarkerError::InputMismatch(e.to_string()))?;

                    let response_text = response
                        .text()
                        .await
                        .map_err(|e| MarkerError::InputMismatch(e.to_string()))?;
                    let response =
                        serde_json::from_str::<GeminiResponse>(&response_text).map_err(|e| {
                            MarkerError::InputMismatch(format!(
                                "error decoding response body: {}. Full response: {}",
                                e, response_text
                            ))
                        })?;

                    if let Some(candidate) = response.candidates.get(0) {
                        if let Some(part) = candidate.content.parts.get(0) {
                            part.text.clone()
                        } else {
                            "Could not generate AI feedback.".to_string()
                        }
                    } else {
                        "Could not generate AI feedback.".to_string()
                    }
                };

                feedback_entries.push(FeedbackEntry {
                    task: result.name.clone(),
                    message,
                });
            }

            Ok(feedback_entries)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TaskResult;

    #[tokio::test]
    #[ignore]
    async fn test_ai_feedback_generation() {
        let feedback_generator = AiFeedback;
        let task_results = vec![
            TaskResult {
                name: "Calculate factorial".to_string(),
                matched_patterns: vec![],
                missed_patterns: vec![
                    "Handles zero".to_string(),
                    "Handles positive numbers".to_string(),
                ],
                awarded: 0,
                possible: 10,
            },
            TaskResult {
                name: "Check for palindrome".to_string(),
                matched_patterns: vec!["Handles 'racecar'".to_string()],
                missed_patterns: vec![],
                awarded: 5,
                possible: 5,
            },
        ];

        let feedback = feedback_generator
            .assemble_feedback(&task_results)
            .await
            .unwrap();

        assert_eq!(feedback.len(), 2);

        let factorial_feedback = &feedback[0];
        assert_eq!(factorial_feedback.task, "Calculate factorial");
        assert!(!factorial_feedback.message.to_lowercase().contains("answer"));
        assert!(!factorial_feedback.message.contains("All patterns matched"));
        assert!(
            !factorial_feedback
                .message
                .contains("Could not generate AI feedback.")
        );

        let palindrome_feedback = &feedback[1];
        assert_eq!(palindrome_feedback.task, "Check for palindrome");
        assert_eq!(palindrome_feedback.message, "All patterns matched");
    }
}
