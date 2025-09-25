use serde::Serialize;
use util::ws::WebSocketManager;

use super::payload;
use crate::ws::core::{envelope, event::Event};

#[derive(Debug, Serialize)]
pub struct SessionUpdatedEvent {
    #[serde(flatten)]
    pub payload: payload::SessionUpdated,
}
impl Event for SessionUpdatedEvent {
    const NAME: &'static str = "attendance.session_updated";
    fn topic_path(&self) -> String {
        format!("attendance:session:{}", self.payload.session_id)
    }
}

#[derive(Debug, Serialize)]
pub struct AttendanceMarkedEvent {
    #[serde(flatten)]
    pub payload: payload::AttendanceMarked,
}
impl Event for AttendanceMarkedEvent {
    const NAME: &'static str = "attendance.marked";
    fn topic_path(&self) -> String {
        format!("attendance:session:{}", self.payload.session_id)
    }
}

#[derive(Debug, Serialize)]
pub struct SessionDeletedEvent {
    #[serde(flatten)]
    pub payload: payload::SessionDeleted,
}
impl Event for SessionDeletedEvent {
    const NAME: &'static str = "attendance.session_deleted";
    fn topic_path(&self) -> String {
        format!("attendance:session:{}", self.payload.session_id)
    }
}

/* ---------- one-liner helpers ---------- */

pub async fn session_updated(ws: &WebSocketManager, p: payload::SessionUpdated) {
    envelope::emit(ws, &SessionUpdatedEvent { payload: p }).await;
}

pub async fn attendance_marked(ws: &WebSocketManager, p: payload::AttendanceMarked) {
    envelope::emit(ws, &AttendanceMarkedEvent { payload: p }).await;
}

pub async fn session_deleted(ws: &WebSocketManager, p: payload::SessionDeleted) {
    envelope::emit(ws, &SessionDeletedEvent { payload: p }).await;
}
