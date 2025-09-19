use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct AttendanceSessionResponse {
    pub id: i64,
    pub module_id: i64,
    pub created_by: i64,
    pub title: String,
    pub active: bool,
    pub rotation_seconds: i32,
    pub restrict_by_ip: bool,
    pub allowed_ip_cidr: Option<String>,
    pub created_from_ip: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub attended_count: i64, // students who marked for this session
    pub student_count: i64,  // total students in module
}

impl From<db::models::attendance_session::Model> for AttendanceSessionResponse {
    fn from(m: db::models::attendance_session::Model) -> Self {
        Self {
            id: m.id,
            module_id: m.module_id,
            created_by: m.created_by,
            title: m.title,
            active: m.active,
            rotation_seconds: m.rotation_seconds,
            restrict_by_ip: m.restrict_by_ip,
            allowed_ip_cidr: m.allowed_ip_cidr,
            created_from_ip: m.created_from_ip,
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
            attended_count: 0,
            student_count: 0,
        }
    }
}

impl AttendanceSessionResponse {
    pub fn from_with_counts(
        m: db::models::attendance_session::Model,
        attended_count: i64,
        student_count: i64,
    ) -> Self {
        let mut base = Self::from(m);
        base.attended_count = attended_count;
        base.student_count = student_count;
        base
    }
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub q: Option<String>,    // search in title
    pub active: Option<bool>, // filter by current status
    pub sort: Option<String>, // "created_at", "-created_at", "title", "-title"
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub sessions: Vec<AttendanceSessionResponse>,
    pub page: i32,
    pub per_page: i32,
    pub total: i32,
}

#[derive(Deserialize)]
pub struct CreateSessionReq {
    pub title: String,
    pub active: Option<bool>, // ← NEW
    pub rotation_seconds: Option<i32>,
    pub code_length: Option<i16>,
    pub restrict_by_ip: Option<bool>,
    pub allowed_ip_cidr: Option<String>,
    pub pin_to_creator_ip: Option<bool>,
    pub allow_manual_entry: Option<bool>,
}

#[derive(Deserialize)]
pub struct EditSessionReq {
    pub title: Option<String>,
    pub active: Option<bool>, // ← NEW
    pub rotation_seconds: Option<i32>,
    pub code_length: Option<i16>,
    pub restrict_by_ip: Option<bool>,
    pub allowed_ip_cidr: Option<String>,
    pub created_from_ip: Option<String>,
    pub allow_manual_entry: Option<bool>,
}
