// api/src/ws/submissions/payload.rs
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct MarkSummary {
    pub earned: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubmissionStatusPayload {
    pub module_id: i64,
    pub assignment_id: i64,
    pub submission_id: i64,
    pub username: Option<String>, // best-effort lookup; None if not found
    pub attempt: i64,
    pub status: String, // serialize of your SubmissionStatus
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark: Option<MarkSummary>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubmissionNewPayload {
    pub module_id: i64,
    pub assignment_id: i64,
    pub submission_id: i64,
    pub username: Option<String>,
    pub attempt: i64,
    pub is_practice: bool,
    /// RFC3339
    pub created_at: String,
}
