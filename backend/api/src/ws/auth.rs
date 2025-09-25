use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};

use crate::auth::claims::AuthUser;
use crate::auth::guards::{is_superuser, user_has_any_role};
use crate::ws::types::ClientTopic;

use db::models::assignment::{Column as AssCol, Entity as AssEntity};
use db::models::attendance_session::{Column as SessCol, Entity as SessEntity};
use db::models::tickets::{Column as TicCol, Entity as TicEntity};

/// staff sets reused below
const STAFF_ROLES: &[&str] = &["Lecturer", "AssistantLecturer"];
const STAFF_ROLES_WITH_TUTORS: &[&str] = &["Lecturer", "AssistantLecturer", "Tutor"];

/// Result type that lets the caller send a specific reason back to the client.
pub enum TopicAuth {
    Allowed,
    Denied(&'static str),
}

impl TopicAuth {
    #[inline]
    pub fn is_allowed(&self) -> bool {
        matches!(self, TopicAuth::Allowed)
    }
}

/// Main authorization entrypoint.
/// - Fail-closed on any DB error / missing linkage.
/// - Returns `Denied(reason_code)` you can surface in `WsOut::SubscribeOk.rejected`.
pub async fn authorize_topic(
    db: &sea_orm::DatabaseConnection,
    user: &AuthUser,
    topic: &ClientTopic,
) -> TopicAuth {
    match topic {
        // ------------------------
        // System streams
        // ------------------------
        ClientTopic::System => TopicAuth::Allowed, // any authenticated user

        ClientTopic::SystemAdmin => {
            if user.0.admin {
                TopicAuth::Allowed
            } else {
                TopicAuth::Denied("admin_only")
            }
        }

        // ------------------------
        // Attendance (instructor-only per module)
        // ------------------------
        ClientTopic::AttendanceSession { session_id } => {
            let sid = *session_id;
            match module_id_for_session(db, sid).await {
                Some(module_id) => {
                    if user.0.admin || is_superuser(user.0.sub).await {
                        TopicAuth::Allowed
                    } else if user_has_any_role(db, user.0.sub, module_id, STAFF_ROLES).await {
                        TopicAuth::Allowed
                    } else {
                        TopicAuth::Denied("not_module_staff")
                    }
                }
                None => TopicAuth::Denied("session_not_found"),
            }
        }

        // ------------------------
        // Ticket chat: author OR module staff (incl. tutors) OR admin/superuser
        // ------------------------
        ClientTopic::TicketChat { ticket_id } => {
            let tid = *ticket_id;
            match ticket_owner_and_module(db, tid).await {
                Some((owner_id, module_id)) => {
                    if user.0.admin || is_superuser(user.0.sub).await {
                        TopicAuth::Allowed
                    } else if user.0.sub == owner_id {
                        TopicAuth::Allowed
                    } else if user_has_any_role(db, user.0.sub, module_id, STAFF_ROLES_WITH_TUTORS)
                        .await
                    {
                        TopicAuth::Allowed
                    } else {
                        TopicAuth::Denied("not_allowed_for_ticket")
                    }
                }
                None => TopicAuth::Denied("ticket_not_found"),
            }
        }

        // ------------------------
        // Assignment submissions (staff aggregate)
        // ------------------------
        ClientTopic::AssignmentSubmissionsStaff { assignment_id } => {
            let aid = *assignment_id;
            match module_id_for_assignment(db, aid).await {
                Some(module_id) => {
                    if user.0.admin || is_superuser(user.0.sub).await {
                        TopicAuth::Allowed
                    } else if user_has_any_role(db, user.0.sub, module_id, STAFF_ROLES_WITH_TUTORS)
                        .await
                    {
                        TopicAuth::Allowed
                    } else {
                        TopicAuth::Denied("not_module_staff")
                    }
                }
                None => TopicAuth::Denied("assignment_not_found"),
            }
        }

        // ------------------------
        // Assignment submissions (owner-only)
        // STRICT: only the owner; admins & staff are not allowed.
        // ------------------------
        ClientTopic::AssignmentSubmissionsOwner {
            assignment_id,
            user_id,
        } => {
            let aid = *assignment_id;
            let uid = *user_id;

            if !assignment_exists(db, aid).await {
                return TopicAuth::Denied("assignment_not_found");
            }
            if user.0.sub == uid {
                TopicAuth::Allowed
            } else {
                TopicAuth::Denied("not_owner")
            }
        }
    }
}

/* --------------- helpers (column-only selects, fail-closed) --------------- */

async fn module_id_for_session(db: &sea_orm::DatabaseConnection, session_id: i64) -> Option<i64> {
    SessEntity::find()
        .select_only()
        .column(SessCol::ModuleId)
        .filter(SessCol::Id.eq(session_id))
        .into_tuple::<i64>()
        .one(db)
        .await
        .ok()?
}

async fn module_id_for_assignment(
    db: &sea_orm::DatabaseConnection,
    assignment_id: i64,
) -> Option<i64> {
    AssEntity::find()
        .select_only()
        .column(AssCol::ModuleId)
        .filter(AssCol::Id.eq(assignment_id))
        .into_tuple::<i64>()
        .one(db)
        .await
        .ok()?
}

async fn assignment_exists(db: &sea_orm::DatabaseConnection, assignment_id: i64) -> bool {
    AssEntity::find()
        .select_only()
        .column(AssCol::Id)
        .filter(AssCol::Id.eq(assignment_id))
        .into_tuple::<i64>()
        .one(db)
        .await
        .ok()
        .flatten()
        .is_some()
}

/// returns (ticket_owner_id, module_id)
async fn ticket_owner_and_module(
    db: &sea_orm::DatabaseConnection,
    ticket_id: i64,
) -> Option<(i64, i64)> {
    // 1) get (user_id, assignment_id) from ticket
    let (owner_id, assignment_id) = TicEntity::find()
        .select_only()
        .column(TicCol::UserId)
        .column(TicCol::AssignmentId)
        .filter(TicCol::Id.eq(ticket_id))
        .into_tuple::<(i64, i64)>()
        .one(db)
        .await
        .ok()??;

    // 2) map assignment -> module_id
    let module_id = module_id_for_assignment(db, assignment_id).await?;
    Some((owner_id, module_id))
}
