//! Ticket message utilities.
//!
//! This module provides helper types and conversions for ticket message endpoints.
//!
//! It includes:
//! - `MessageResponse`: a serializable response type for ticket message API endpoints.
//! - `UserResponse`: embedded user information for message responses.

use services::ticket_message::TicketMessage;

/// Represents a user in the context of a ticket message.
#[derive(serde::Serialize)]
pub struct UserResponse {
    /// User ID
    pub id: i64,
    /// Username
    pub username: String,
}

/// Represents a ticket message along with its author information.
#[derive(serde::Serialize)]
pub struct MessageResponse {
    /// Message ID
    pub id: i64,
    /// Ticket ID this message belongs to
    pub ticket_id: i64,
    /// Content of the message
    pub content: String,
    /// Creation timestamp in RFC3339 format
    pub created_at: String,
    /// Last update timestamp in RFC3339 format
    pub updated_at: String,
    /// Optional user info for the author of the message
    pub user: Option<UserResponse>,
}

impl From<(TicketMessage, String)> for MessageResponse {
    fn from((message, username): (TicketMessage, String)) -> Self {
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
