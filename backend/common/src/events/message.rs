use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventMessage {
    Progress { percent: u8 },
    Status { message: String },
    Finished { success: bool },
    Error { message: String },
    LogLine { line: String },
}