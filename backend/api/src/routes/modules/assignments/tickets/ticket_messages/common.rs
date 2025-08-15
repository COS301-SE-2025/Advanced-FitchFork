use db::models::ticket_messages::Model as TicketMessageModel;

#[derive(serde::Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
}

#[derive(serde::Serialize)]
pub struct MessageResponse {
    pub id: i64,
    pub ticket_id: i64,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub user: Option<UserResponse>,
}

impl From<(TicketMessageModel, String)> for MessageResponse {
    fn from((message, username): (TicketMessageModel, String)) -> Self {
        Self {
            id: message.id,
            ticket_id: message.ticket_id,
            content: message.content,
            created_at: message.created_at.to_rfc3339(),
            updated_at: message.updated_at.to_rfc3339(),
            user: Some(UserResponse {
                id: message.user_id,
                username,
            }),
        }
    }
}
