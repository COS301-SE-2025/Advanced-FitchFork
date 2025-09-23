use crate::service::{AppError, Service, ToActiveModel};
use chrono::{DateTime, Utc};
use db::{
    models::attendance_session::{ActiveModel, Column, Entity},
    repository::Repository,
};
use hmac::Hmac;
use sea_orm::{DbErr, IntoActiveModel, Set};
use sha2::Sha256;
use std::future::Future;
use std::pin::Pin;
use std::{env, fs, path::PathBuf};

pub use db::models::attendance_session::Model as AttendanceSession;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct CreateAttendanceSession {
    pub module_id: i64,
    pub created_by: i64,
    pub title: String,
    pub active: bool,
    pub rotation_seconds: i32,
    pub restrict_by_ip: bool,
    pub allowed_ip_cidr: Option<String>,
    pub created_from_ip: Option<String>,
    pub secret_hex: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateAttendanceSession {
    pub id: i64,
}

impl ToActiveModel<Entity> for CreateAttendanceSession {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let secret = match secret_hex {
            Some(s) => s.to_owned(),
            None => {
                use rand::RngCore;
                let mut buf = [0u8; 32];
                rand::rngs::OsRng.fill_bytes(&mut buf);
                hex::encode(buf)
            }
        };

        Ok(ActiveModel {
            module_id: Set(module_id),
            created_by: Set(created_by),
            title: Set(title.to_owned()),
            active: Set(active),
            rotation_seconds: Set(rotation_seconds),
            secret: Set(secret),
            restrict_by_ip: Set(restrict_by_ip),
            allowed_ip_cidr: Set(allowed_ip_cidr.map(|s| s.to_owned())),
            created_from_ip: Set(created_from_ip.map(|s| s.to_owned())),
            ..Default::default()
        })
    }
}

impl ToActiveModel<Entity> for UpdateAttendanceSession {
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

pub struct AttendanceSessionService;

impl<'a> Service<'a, Entity, Column, CreateAttendanceSession, UpdateAttendanceSession>
    for AttendanceSessionService
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

impl AttendanceSessionService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

    #[inline]
    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn window(&self, now: DateTime<Utc>) -> i64 {
        let secs = now.timestamp();
        let r = i64::from(self.rotation_seconds.max(1));
        secs.div_euclid(r)
    }

    pub fn code_for_window(&self, window: i64) -> String {
        const DIGITS: u32 = 6; // fixed length now
        let mut mac = HmacSha256::new_from_slice(self.secret.as_bytes()).expect("HMAC key");
        mac.update(&window.to_be_bytes());
        let digest = mac.finalize().into_bytes();

        let offset = (digest[31] & 0x0f) as usize;
        let slice = &digest[offset..offset + 4];
        let val = u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]) & 0x7fff_ffff;

        let modulus = 10u32.pow(DIGITS);
        let num = val % modulus;

        let mut s = num.to_string();
        while s.len() < DIGITS as usize {
            s.insert(0, '0');
        }
        s
    }

    pub fn current_code(&self, now: DateTime<Utc>) -> String {
        self.code_for_window(self.window(now))
    }

    pub fn ip_permitted(&self, client_ip: Option<&str>) -> bool {
        if !self.restrict_by_ip {
            return true;
        }
        let Some(ip) = client_ip else {
            return false;
        };
        if let Some(exact) = &self.created_from_ip {
            return ip == exact;
        }
        if let Some(_cidr) = &self.allowed_ip_cidr {
            return true; // TODO: implement CIDR match
        }
        false
    }

    // --- utilities unchanged (student counts) ---
    pub async fn student_count_for_module<C>(db: &C, module_id: i64) -> Result<i64, DbErr>
    where
        C: ConnectionTrait,
    {
        let c = UmrEntity::find()
            .filter(UmrCol::ModuleId.eq(module_id))
            .filter(UmrCol::Role.eq(UmrRole::Student))
            .count(db)
            .await?;
        Ok(c)
    }

    pub async fn attended_student_count<C>(
        db: &C,
        module_id: i64,
        session_id: i64,
    ) -> Result<i64, DbErr>
    where
        C: ConnectionTrait,
    {
        let student_ids_subq = UmrEntity::find()
            .select_only()
            .column(UmrCol::UserId)
            .filter(UmrCol::ModuleId.eq(module_id))
            .filter(UmrCol::Role.eq(UmrRole::Student))
            .into_query();

        let c = attendance_record::Entity::find()
            .filter(attendance_record::Column::SessionId.eq(session_id))
            .filter(attendance_record::Column::UserId.in_subquery(student_ids_subq))
            .count(db)
            .await?;
        Ok(c)
    }

    pub async fn attended_student_counts_for<C>(
        db: &C,
        module_id: i64,
        session_ids: &[i64],
    ) -> Result<HashMap<i64, i64>, DbErr>
    where
        C: ConnectionTrait,
    {
        if session_ids.is_empty() {
            return Ok(HashMap::new());
        }

        #[derive(FromQueryResult)]
        struct Row {
            session_id: i64,
            cnt: i64,
        }

        let student_ids_subq = UmrEntity::find()
            .select_only()
            .column(UmrCol::UserId)
            .filter(UmrCol::ModuleId.eq(module_id))
            .filter(UmrCol::Role.eq(UmrRole::Student))
            .into_query();

        let rows: Vec<Row> = attendance_record::Entity::find()
            .select_only()
            .column(attendance_record::Column::SessionId)
            .column_as(
                Expr::expr(Func::count(Expr::col(attendance_record::Column::UserId))),
                "cnt",
            )
            .filter(attendance_record::Column::UserId.in_subquery(student_ids_subq))
            .filter(attendance_record::Column::SessionId.is_in(session_ids.iter().cloned()))
            .group_by(attendance_record::Column::SessionId)
            .into_model::<Row>()
            .all(db)
            .await?;

        Ok(rows.into_iter().map(|r| (r.session_id, r.cnt)).collect())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::models::{module, user};
//     use crate::test_utils::setup_test_db;
//     use chrono::{TimeZone, Utc};

//     #[tokio::test]
//     async fn test_session_code_rotates() {
//         let db = setup_test_db().await;

//         // --- seed FK dependencies: a user and a module ---
//         let lecturer = user::Model::create(&db, "lect1", "lect1@test.com", "pw", false)
//             .await
//             .expect("create lecturer");
//         let m = module::Model::create(&db, "COS101", 2025, Some("Test Module"), 16)
//             .await
//             .expect("create module");

//         // --- create the session using real FK ids ---
//         let s = Model::create(
//             &db,
//             m.id,        // module_id
//             lecturer.id, // created_by
//             "Lecture 5",
//             true,  // active
//             30,    // rotation_seconds
//             false, // restrict_by_ip
//             None,  // allowed_ip_cidr
//             None,  // created_from_ip
//             Some("00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"),
//         )
//         .await
//         .unwrap();

//         // codes should differ across a window boundary
//         let t1 = Utc.with_ymd_and_hms(2025, 9, 8, 10, 0, 14).unwrap(); // window N
//         let t2 = Utc.with_ymd_and_hms(2025, 9, 8, 10, 0, 31).unwrap(); // window N+1
//         assert_ne!(s.current_code(t1), s.current_code(t2));
//     }
// }
