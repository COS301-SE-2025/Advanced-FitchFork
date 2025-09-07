use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TicketIncoming {
    /// Typing indicator from client
    Typing { sender: String },
    /// Keepalive ping from client (app-level) — note: framework also supports {"type":"ping"}
    Ping,
}
