use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SessionUpdated {
    pub session_id: i64,
    pub active: bool,
    pub rotation_seconds: i32,
    pub title: String,
    pub restrict_by_ip: bool,
    pub allowed_ip_cidr: Option<String>,
    pub created_from_ip: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AttendanceMarked {
    pub session_id: i64,
    pub user_id: i64,
    pub taken_at: String, // RFC3339
    pub count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>, // e.g. "admin_manual"
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionDeleted {
    pub session_id: i64,
}
