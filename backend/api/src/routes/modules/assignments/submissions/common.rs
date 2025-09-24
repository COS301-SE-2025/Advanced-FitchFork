//! Submission utilities and response types.
//!
//! This module provides types and helpers for submissions-related endpoints.
//!
//! It includes:
//! - `ListSubmissionsQuery` → query parameters for paginated submission listings
//! - `UserResponse` → represents a user associated with a submission
//! - `Mark` → represents earned/total marks for a submission
//! - `SubmissionListItem` → single item in a submissions list
//! - `SubmissionsListResponse` → paginated submissions list
//! - `SubmissionResponse` → minimal response for a single submission
//! - `MarkSummary` → summary of earned vs total marks
//! - `SubmissionDetailResponse` → detailed response after grading a submission

use serde::{Deserialize, Serialize};

/// Query parameters for submissions listing endpoints.
#[derive(Debug, Deserialize)]
pub struct ListSubmissionsQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort: Option<String>,
    pub query: Option<String>,
    pub username: Option<String>,
    pub late: Option<bool>,
    pub ignored: Option<bool>,
}

/// Represents a user associated with a submission.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
}

/// Represents the mark of a submission.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mark {
    pub earned: f32,
    pub total: f32,
}

/// Single item in a submissions list response.
#[derive(Debug, Serialize)]
pub struct SubmissionListItem {
    pub id: i64,
    pub user: UserResponse,
    pub attempt: i64,
    pub filename: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_practice: bool,
    pub is_late: bool,
    pub mark: Option<Mark>,
    pub ignored: bool,
    pub status: String,
}

/// Paginated response of submissions list.
#[derive(Debug, Serialize)]
pub struct SubmissionsListResponse {
    pub submissions: Vec<SubmissionListItem>,
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
}

/// Minimal submission response for single submission detail.
#[derive(Debug, Serialize)]
pub struct SubmissionResponse {
    pub id: i64,
    pub attempt: i64,
    pub filename: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_late: bool,
    pub is_practice: bool,
    pub mark: Option<Mark>,
    pub ignored: bool,
    pub status: String,
}

/// Represents a summary of earned vs total marks.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarkSummary {
    pub earned: f64,
    pub total: f64,
}

/// Represents code coverage for a submission.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeCoverageFile {
    pub path: String,
    pub earned: f64,
    pub total: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeCoverage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<MarkSummary>,
    pub files: Vec<CodeCoverageFile>,
}

/// Detailed response returned after grading a submission.
#[derive(Debug, Serialize, Deserialize)]
pub struct SubmissionDetailResponse {
    pub id: i64,
    pub attempt: i64,
    pub filename: String,
    pub hash: String,
    pub created_at: String,
    pub updated_at: String,
    pub mark: MarkSummary,
    pub is_practice: bool,
    pub is_late: bool,
    pub ignored: bool,
    pub status: String,
    pub tasks: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_coverage: Option<CodeCoverage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<serde_json::Value>,
    pub plagiarism: PlagiarismInfo,
}

// ---- instant ACK (client will GET /submissions/{id} and attach WS) ----
#[derive(serde::Serialize)]
pub struct SubmitAck {
    pub id: i64,
    pub status: String, // e.g. "queued", "failed_upload"
    pub attempt: i64,
    pub is_practice: bool,
    pub filename: String,
    pub hash: String,
    pub created_at: String, // RFC3339
}

/// Represents plagiarism-related information for a submission.
#[derive(Debug, Serialize, Deserialize)]
pub struct PlagiarismInfo {
    pub flagged: bool,
    pub similarity: f32,
    pub lines_matched: i64,
    pub description: String,
}
