use serde::{Deserialize, Serialize};

/// Query parameters for submissions listing endpoints.
/// Supports pagination, search query, sorting, and optional username filter.
#[derive(Debug, Deserialize)]
pub struct ListSubmissionsQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort: Option<String>,
    pub query: Option<String>,
    pub username: Option<String>,
    pub late: Option<bool>,
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
/// Includes grading marks, flags, and optional coverage/complexity reports.
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