use sea_orm::entity::prelude::*;
use sea_orm::{Set, ActiveModelTrait, ConnectionTrait, QueryFilter, ColumnTrait};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize)]
#[sea_orm(table_name = "attendance_records")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub session_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i64,

    pub taken_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    // REMOVED: method
    pub token_window: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::attendance_session::Entity", from = "Column::SessionId", to = "super::attendance_session::Column::Id")]
    Session,
    #[sea_orm(belongs_to = "super::user::Entity", from = "Column::UserId", to = "super::user::Column::Id")]
    User,
}

impl Related<super::attendance_session::Entity> for Entity {
    fn to() -> RelationDef { Relation::Session.def() }
    fn via() -> Option<RelationDef> { None }
}
impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef { Relation::User.def() }
    fn via() -> Option<RelationDef> { None }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Mark attendance after verifying code/IP and that the session is active.
    pub async fn mark<C>(
        db: &C,
        session: &super::attendance_session::Model,
        user_id: i64,
        submitted_code: &str,
        client_ip: Option<&str>,
        now: DateTime<Utc>,
        window_tolerance: i64,
    ) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !session.is_active() {
            return Err(DbErr::Custom("Attendance session is not active".into()));
        }

        // manual is always allowed now — no flag; still enforce IP policy if configured
        if !session.ip_permitted(client_ip) {
            return Err(DbErr::Custom("IP not permitted for this session".into()));
        }

        // Verify rotating code with tolerance
        let w = session.window(now);
        let mut valid = false;
        let mut accepted_window = w;

        for delta in -window_tolerance..=window_tolerance {
            let ww = w + delta;
            if session.code_for_window(ww) == submitted_code.trim() {
                valid = true;
                accepted_window = ww;
                break;
            }
        }
        if !valid {
            return Err(DbErr::Custom("Invalid or expired attendance code".into()));
        }

        // Ensure not already marked
        if Entity::find()
            .filter(Column::SessionId.eq(session.id))
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await?
            .is_some()
        {
            return Err(DbErr::Custom("Attendance already recorded".into()));
        }

        let am = ActiveModel {
            session_id: Set(session.id),
            user_id: Set(user_id),
            taken_at: Set(now),
            ip_address: Set(client_ip.map(|s| s.to_owned())),
            token_window: Set(accepted_window),
        };
        am.insert(db).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::attendance_session;
    use crate::test_utils::setup_test_db;
    use chrono::{TimeZone, Utc};

    #[tokio::test]
    async fn test_mark_attendance_once() {
        let db = setup_test_db().await;

        let sess = attendance_session::Model::create(
            &db,
            1, 1, "Lec",
            true, 30,
            false, None, None,
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        ).await.unwrap();

        let now = Utc.with_ymd_and_hms(2025, 9, 8, 10, 15, 2).unwrap();
        let code = sess.current_code(now);

        let rec = Model::mark(&db, &sess, 42, &code, Some("203.0.113.5"), now, 1).await.unwrap();
        assert_eq!(rec.session_id, sess.id);
        assert_eq!(rec.user_id, 42);

        let dup = Model::mark(&db, &sess, 42, &code, Some("203.0.113.5"), now, 1).await;
        assert!(dup.is_err());
    }
}
