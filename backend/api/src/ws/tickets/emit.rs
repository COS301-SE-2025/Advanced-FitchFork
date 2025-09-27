// api/src/ws/tickets/emit.rs
use serde::Serialize;
use util::ws::WebSocketManager;

use crate::ws::core::{envelope, event::Event};
use crate::ws::types::ClientTopic;

use super::payload;

/* ------------ Events (typed, stable names) ------------ */

#[derive(Debug, Serialize)]
pub struct TicketMessageCreated {
    #[serde(flatten)]
    pub payload: payload::Message,
    #[serde(skip)]
    pub ticket_id: i64,
}
impl Event for TicketMessageCreated {
    const NAME: &'static str = "ticket.message_created";
    fn topic_path(&self) -> String {
        ClientTopic::TicketChat {
            ticket_id: self.ticket_id,
        }
        .path()
    }
}

#[derive(Debug, Serialize)]
pub struct TicketMessageUpdated {
    #[serde(flatten)]
    pub payload: payload::Message,
    #[serde(skip)]
    pub ticket_id: i64,
}
impl Event for TicketMessageUpdated {
    const NAME: &'static str = "ticket.message_updated";
    fn topic_path(&self) -> String {
        ClientTopic::TicketChat {
            ticket_id: self.ticket_id,
        }
        .path()
    }
}

#[derive(Debug, Serialize)]
pub struct TicketMessageDeleted {
    #[serde(flatten)]
    pub payload: payload::MessageId,
    #[serde(skip)]
    pub ticket_id: i64,
}
impl Event for TicketMessageDeleted {
    const NAME: &'static str = "ticket.message_deleted";
    fn topic_path(&self) -> String {
        ClientTopic::TicketChat {
            ticket_id: self.ticket_id,
        }
        .path()
    }
}

/* ------------ One-liner emit helpers ------------ */

pub async fn message_created(ws: &WebSocketManager, msg: payload::Message) {
    let ev = TicketMessageCreated {
        ticket_id: msg.ticket_id,
        payload: msg,
    };
    envelope::emit(ws, &ev).await;
}

pub async fn message_updated(ws: &WebSocketManager, msg: payload::Message) {
    let ev = TicketMessageUpdated {
        ticket_id: msg.ticket_id,
        payload: msg,
    };
    envelope::emit(ws, &ev).await;
}

pub async fn message_deleted(ws: &WebSocketManager, ticket_id: i64, id: i64) {
    let ev = TicketMessageDeleted {
        ticket_id,
        payload: payload::MessageId { id },
    };
    envelope::emit(ws, &ev).await;
}
