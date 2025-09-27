// api/src/ws/tickets/payload.rs
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct LightUser {
    pub id: i64,
    pub username: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub id: i64,
    pub ticket_id: i64,
    pub content: String,
    pub created_at: String, // RFC3339
    pub updated_at: String, // RFC3339
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<LightUser>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageId {
    pub id: i64,
}
