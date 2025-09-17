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
//! - `CodeComplexitySummary` → summary of code complexity metrics
//! - `CodeComplexity` → detailed code complexity information
//! - `SubmissionDetailResponse` → detailed response after grading a submission

use serde::{Deserialize, Serialize};

/// Query parameters for submissions listing endpoints.
#[derive(Debug, Deserialize)]
pub struct ListSubmissionsQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub sort: Option<String>,
    pub query: Option<String>,
    pub username: Option<String>,
    pub late: Option<bool>,
    pub ignored: Option<bool>,
}

/// Represents a user associated with a submission.
#[derive(Debug, Serialize, Clone)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
}

/// Represents the mark of a submission.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mark {
    pub earned: i64,
    pub total: i64,
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
}

/// Paginated response of submissions list.
#[derive(Debug, Serialize)]
pub struct SubmissionsListResponse {
    pub submissions: Vec<SubmissionListItem>,
    pub page: u64,
    pub per_page: u64,
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
}

/// Represents a summary of earned vs total marks.
#[derive(Debug, Serialize, Deserialize)]
pub struct MarkSummary {
    pub earned: i64,
    pub total: i64,
}

/// Represents a summary of code complexity metrics.
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeComplexitySummary {
    pub earned: i64,
    pub total: i64,
}

/// Represents code complexity details, including per-metric results.
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeComplexity {
    pub summary: CodeComplexitySummary,
    pub metrics: Vec<serde_json::Value>,
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
    pub tasks: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_coverage: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_complexity: Option<CodeComplexity>,
}
