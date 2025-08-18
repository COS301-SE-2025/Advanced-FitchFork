
pub fn ticket_chat_topic(ticket_id: i64) -> String {
    format!("ws/tickets/{ticket_id}")
}
