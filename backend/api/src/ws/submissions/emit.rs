// api/src/ws/submissions/emit.rs
use serde::Serialize;
use util::ws::WebSocketManager;

use super::payload::SubmissionStatusPayload;
use crate::ws::core::{envelope, event::Event};
use crate::ws::submissions::payload::SubmissionNewPayload;
use crate::ws::types::ClientTopic;

/* =========================
EVENTS
========================= */

#[derive(Debug, Serialize)]
pub struct SubmissionStatusOwnerEvent {
    #[serde(flatten)]
    pub payload: SubmissionStatusPayload,
    #[serde(skip)]
    pub assignment_id: i64,
    #[serde(skip)]
    pub user_id: i64,
}

impl Event for SubmissionStatusOwnerEvent {
    const NAME: &'static str = "submission.status";
    fn topic_path(&self) -> String {
        ClientTopic::AssignmentSubmissionsOwner {
            assignment_id: self.assignment_id,
            user_id: self.user_id,
        }
        .path()
    }
}

#[derive(Debug, Serialize)]
pub struct SubmissionStatusStaffEvent {
    #[serde(flatten)]
    pub payload: SubmissionStatusPayload,
    #[serde(skip)]
    pub assignment_id: i64,
}

impl Event for SubmissionStatusStaffEvent {
    const NAME: &'static str = "submission.status";
    fn topic_path(&self) -> String {
        ClientTopic::AssignmentSubmissionsStaff {
            assignment_id: self.assignment_id,
        }
        .path()
    }
}

#[derive(Debug, Serialize)]
pub struct SubmissionNewStaffEvent {
    #[serde(flatten)]
    pub payload: SubmissionNewPayload,
    #[serde(skip)]
    pub assignment_id: i64,
}

impl Event for SubmissionNewStaffEvent {
    // Event name clients will match on
    const NAME: &'static str = "submission.new_submission";
    fn topic_path(&self) -> String {
        ClientTopic::AssignmentSubmissionsStaff {
            assignment_id: self.assignment_id,
        }
        .path()
    }
}

/* =========================
HELPERS
========================= */

/// Emit a status event to the *owner* (single user) topic.
pub async fn status_owner(
    ws: &WebSocketManager,
    assignment_id: i64,
    user_id: i64,
    payload: SubmissionStatusPayload,
) {
    let ev_owner = SubmissionStatusOwnerEvent {
        payload,
        assignment_id,
        user_id,
    };
    envelope::emit(ws, &ev_owner).await;
}

/// Emit a status event to the *staff* aggregate topic.
pub async fn status_staff(
    ws: &WebSocketManager,
    assignment_id: i64,
    payload: SubmissionStatusPayload,
) {
    let ev_staff = SubmissionStatusStaffEvent {
        payload,
        assignment_id,
    };
    envelope::emit(ws, &ev_staff).await;
}

/// One call emits to both topics (owner + staff).
pub async fn status(
    ws: &WebSocketManager,
    owner_assignment_id: i64,
    owner_user_id: i64,
    shared_payload: SubmissionStatusPayload,
) {
    // Owner
    status_owner(
        ws,
        owner_assignment_id,
        owner_user_id,
        shared_payload.clone(),
    )
    .await;

    // Staff
    status_staff(ws, owner_assignment_id, shared_payload).await;
}

/// Emit a "new submission created" event to the staff stream only.
/// This is NOT emitted for resubmissions; only on the initial creation.
pub async fn new_submission_staff(
    ws: &WebSocketManager,
    assignment_id: i64,
    payload: SubmissionNewPayload,
) {
    let ev = SubmissionNewStaffEvent {
        assignment_id,
        payload,
    };
    envelope::emit(ws, &ev).await;
}
