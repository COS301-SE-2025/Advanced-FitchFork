use serde::Serialize;
use util::ws::WebSocketManager;

use crate::ws::core::{envelope, event::Event};
use crate::ws::system::payload::{SystemHealthAdminPayload, SystemHealthGeneralPayload};
use crate::ws::types::ClientTopic;

/* =========================
EVENTS
========================= */

/// General system health (all authenticated users)
#[derive(Debug, Serialize)]
pub struct SystemHealthGeneral {
    #[serde(flatten)]
    pub payload: SystemHealthGeneralPayload,
}

impl Event for SystemHealthGeneral {
    const NAME: &'static str = "system.health";
    fn topic_path(&self) -> String {
        ClientTopic::System.path()
    }
}

/// Admin system health (admins & superusers only)
#[derive(Debug, Serialize)]
pub struct SystemHealthAdmin {
    #[serde(flatten)]
    pub payload: SystemHealthAdminPayload,
}

impl Event for SystemHealthAdmin {
    const NAME: &'static str = "system.health_admin";
    fn topic_path(&self) -> String {
        ClientTopic::SystemAdmin.path()
    }
}

/* =========================
EMIT HELPERS (ONE-LINERS)
========================= */

pub async fn health_general(ws: &WebSocketManager, payload: SystemHealthGeneralPayload) {
    let ev = SystemHealthGeneral { payload };
    envelope::emit(ws, &ev).await;
}

pub async fn health_admin(ws: &WebSocketManager, payload: SystemHealthAdminPayload) {
    let ev = SystemHealthAdmin { payload };
    envelope::emit(ws, &ev).await;
}
