//! # Feedback Strategies Module
//!
//! This module provides pluggable feedback strategies for the marker system.
//! Each strategy implements the [`Feedback`] trait and produces a list of [`FeedbackEntry`]s
//! based on the marking results. This allows for flexible, extensible feedback generation.
//!
//! ## Available Strategies
//!
//! - [`auto_feedback`]: Generates automatic feedback based on matched/missed patterns in student output.
//! - [`manual_feedback`]: Allows instructors to specify custom/manual feedback for each task.
//! - [`ai_feedback`]: Uses an LLM (Large Language Model) to generate advanced, context-aware feedback.

pub mod ai_feedback;
pub mod auto_feedback;
pub mod manual_feedback;
