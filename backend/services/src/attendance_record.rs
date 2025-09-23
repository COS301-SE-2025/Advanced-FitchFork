use crate::service::{AppError, Service, ToActiveModel};
use chrono::{DateTime, Utc};
use db::{
    models::attendance_record::{ActiveModel, Column, Entity},
    repository::Repository,
};
use sea_orm::{DbErr, IntoActiveModel, Set};
use std::future::Future;
use std::pin::Pin;
use std::{env, fs, path::PathBuf};

pub use db::models::attendance_record::Model as AttendanceRecord;

#[derive(Debug, Clone)]
pub struct CreateAttendanceRecord {
    pub session: &super::attendance_session::Model,
    pub user_id: i64,
    pub submitted_code: &str,
    pub client_ip: Option<&str>,
    pub now: DateTime<Utc>,
    pub window_tolerance: i64,
}

#[derive(Debug, Clone)]
pub struct UpdateAttendanceRecord {
    pub id: i64,
}

impl ToActiveModel<Entity> for CreateAttendanceRecord {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        // Verify rotating code with tolerance
        let w = params.session.window(params.now);
        let mut valid = false;
        let mut accepted_window = w;

        for delta in -params.window_tolerance..=params.window_tolerance {
            let ww = w + delta;
            if params.session.code_for_window(ww) == params.submitted_code.trim() {
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
            .filter(Column::SessionId.eq(params.session.id))
            .filter(Column::UserId.eq(params.user_id))
            .one(db)
            .await?
            .is_some()
        {
            return Err(DbErr::Custom("Attendance already recorded".into()));
        }

        Ok(ActiveModel {
            session_id: Set(self.session.id),
            user_id: Set(self.user_id),
            taken_at: Set(self.now),
            ip_address: Set(self.client_ip.map(|s| s.to_owned())),
            token_window: Set(accepted_window),
        })
    }
}

impl ToActiveModel<Entity> for UpdateAttendanceRecord {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let record = match Repository::<Entity, Column>::find_by_id(self.id).await {
            Ok(Some(record)) => record,
            Ok(None) => {
                return Err(AppError::from(DbErr::RecordNotFound(format!(
                    "Attendance record ID {} not found",
                    self.id
                ))));
            }
            Err(err) => return Err(AppError::from(err)),
        };

        let active: ActiveModel = record.into();

        Ok(active)
    }
}

pub struct AttendanceRecordService;

impl<'a> Service<'a, Entity, Column, CreateAttendanceRecord, UpdateAttendanceRecord>
    for AttendanceRecordService
{
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓

    fn create(
        params: CreateAssignment,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<<Entity as sea_orm::EntityTrait>::Model, AppError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            if !params.session.is_active() {
                return Err(DbErr::Custom("Attendance session is not active".into()));
            }

            // manual is always allowed now — no flag; still enforce IP policy if configured
            if !params.session.ip_permitted(params.client_ip) {
                return Err(DbErr::Custom("IP not permitted for this session".into()));
            }

            Repository::<Entity, Column>::create(params).await
        })
    }
}

impl AttendanceRecordService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::models::{attendance_session, module, user};
//     use crate::test_utils::setup_test_db;
//     use chrono::{TimeZone, Utc};

//     #[tokio::test]
//     async fn test_mark_attendance_once() {
//         let db = setup_test_db().await;

//         // --- seed FK dependencies ---
//         let lecturer = user::Model::create(&db, "lect1", "lect1@test.com", "pw", false)
//             .await
//             .unwrap();
//         let student = user::Model::create(&db, "stud1", "stud1@test.com", "pw", false)
//             .await
//             .unwrap();
//         let m = module::Model::create(&db, "COS101", 2025, Some("Test Module"), 16)
//             .await
//             .unwrap();

//         // session must reference real module + creator user
//         let sess = attendance_session::Model::create(
//             &db,
//             m.id,        // module_id FK
//             lecturer.id, // created_by / owner FK
//             "Lec",
//             true,  // manual/active flag (as per your signature)
//             30,    // rotation seconds
//             false, // restrict IP?
//             None,  // ip subnet / prefix
//             None,  // location / whatever the option is
//             Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), // salt
//         )
//         .await
//         .unwrap();

//         let now = Utc.with_ymd_and_hms(2025, 9, 8, 10, 15, 2).unwrap();
//         let code = sess.current_code(now);

//         // use real student.id — not 42
//         let rec = Model::mark(&db, &sess, student.id, &code, Some("203.0.113.5"), now, 1)
//             .await
//             .unwrap();
//         assert_eq!(rec.session_id, sess.id);
//         assert_eq!(rec.user_id, student.id);

//         let dup = Model::mark(&db, &sess, student.id, &code, Some("203.0.113.5"), now, 1).await;
//         assert!(dup.is_err());
//     }
// }
