use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsIn {
    Auth {
        token: String,
    },
    Reauth {
        token: String,
    },
    // structured topics
    Subscribe {
        topics: Vec<ClientTopic>,
        since: Option<u64>,
    },
    Unsubscribe {
        topics: Vec<ClientTopic>,
    },
    Ping,
    Command {
        name: String,
        topic: Option<ClientTopic>,
        data: serde_json::Value,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsOut<T = serde_json::Value> {
    Ready {
        policy_version: u64,
        exp: Option<i64>,
    },
    Pong,
    // server returns path strings
    SubscribeOk {
        accepted: Vec<String>,
        rejected: Vec<(String, String)>,
    },
    UnsubscribeOk {
        topics: Vec<String>,
    },
    Event {
        event: String,
        topic: String,
        v: Option<u64>,
        payload: T,
        ts: String,
    },
    Error {
        code: &'static str,
        message: String,
        meta: Option<HashMap<String, String>>,
    },
}

/* ---------- ClientTopic: structured topics ---------- */
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClientTopic {
    // System streams (no tenant)
    System,      // "system"
    SystemAdmin, // "system:admin"

    // Attendance (per-session)
    AttendanceSession { session_id: i64 }, // "attendance:session:{sid}"

    TicketChat { ticket_id: i64 },

    // Submissions
    AssignmentSubmissionsStaff { assignment_id: i64 },
    AssignmentSubmissionsOwner { assignment_id: i64, user_id: i64 },
}

impl ClientTopic {
    pub fn path(&self) -> String {
        match *self {
            ClientTopic::System => "system".to_string(),
            ClientTopic::SystemAdmin => "system:admin".to_string(),
            ClientTopic::AttendanceSession { session_id } => {
                format!("attendance:session:{session_id}")
            }
            ClientTopic::TicketChat { ticket_id } => format!("tickets:{ticket_id}"),
            ClientTopic::AssignmentSubmissionsStaff { assignment_id } => {
                format!("assignment:{assignment_id}.submissions:staff")
            }
            ClientTopic::AssignmentSubmissionsOwner {
                assignment_id,
                user_id,
            } => format!("assignment:{assignment_id}.submissions:user:{user_id}"),
        }
    }
}
