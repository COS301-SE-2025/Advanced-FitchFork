use sea_orm::entity::prelude::*;
use sea_orm::FromQueryResult;
use sea_orm::{Set, ActiveModelTrait, ConnectionTrait, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, sea_query::{Expr, Func}, QueryTrait};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;

use super::{
    attendance_record,
    user_module_role::{Entity as UmrEntity, Column as UmrCol, Role as UmrRole},
};

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize)]
#[sea_orm(table_name = "attendance_sessions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub module_id: i64,
    pub created_by: i64,
    pub title: String,
    pub active: bool,
    pub rotation_seconds: i32,
    pub secret: String,
    pub restrict_by_ip: bool,
    pub allowed_ip_cidr: Option<String>,
    pub created_from_ip: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::module::Entity", from = "Column::ModuleId", to = "super::module::Column::Id")]
    Module,
    #[sea_orm(belongs_to = "super::user::Entity", from = "Column::CreatedBy", to = "super::user::Column::Id")]
    Creator,
    #[sea_orm(has_many = "super::attendance_record::Entity")]
    Records,
}

impl Related<super::module::Entity> for Entity {
    fn to() -> RelationDef { Relation::Module.def() }
    fn via() -> Option<RelationDef> { None }
}

impl Related<super::attendance_record::Entity> for Entity {
    fn to() -> RelationDef { Relation::Records.def() }
    fn via() -> Option<RelationDef> { None }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    #[inline]
    pub fn is_active(&self) -> bool { self.active }

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
        let num = (val % modulus) as u32;

        let mut s = num.to_string();
        while s.len() < DIGITS as usize { s.insert(0, '0'); }
        s
    }

    pub fn current_code(&self, now: DateTime<Utc>) -> String {
        self.code_for_window(self.window(now))
    }

    pub async fn create<C>(
        db: &C,
        module_id: i64,
        created_by: i64,
        title: &str,
        active: bool,
        rotation_seconds: i32,
        restrict_by_ip: bool,
        allowed_ip_cidr: Option<&str>,
        created_from_ip: Option<&str>,
        secret_hex: Option<&str>,
    ) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let secret = match secret_hex {
            Some(s) => s.to_owned(),
            None => {
                use rand::RngCore;
                let mut buf = [0u8; 32];
                rand::rngs::OsRng.fill_bytes(&mut buf);
                hex::encode(buf)
            }
        };

        let am = ActiveModel {
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
        };

        am.insert(db).await
    }

    pub fn ip_permitted(&self, client_ip: Option<&str>) -> bool {
        if !self.restrict_by_ip { return true; }
        let Some(ip) = client_ip else { return false; };
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
    where C: ConnectionTrait {
        let c = UmrEntity::find()
            .filter(UmrCol::ModuleId.eq(module_id))
            .filter(UmrCol::Role.eq(UmrRole::Student))
            .count(db)
            .await?;
        Ok(c as i64)
    }

    pub async fn attended_student_count<C>(db: &C, module_id: i64, session_id: i64) -> Result<i64, DbErr>
    where C: ConnectionTrait {
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
        Ok(c as i64)
    }

    pub async fn attended_student_counts_for<C>(db: &C, module_id: i64, session_ids: &[i64]) -> Result<HashMap<i64, i64>, DbErr>
    where C: ConnectionTrait {
        if session_ids.is_empty() { return Ok(HashMap::new()); }

        #[derive(FromQueryResult)]
        struct Row { session_id: i64, cnt: i64 }

        let student_ids_subq = UmrEntity::find()
            .select_only()
            .column(UmrCol::UserId)
            .filter(UmrCol::ModuleId.eq(module_id))
            .filter(UmrCol::Role.eq(UmrRole::Student))
            .into_query();

        let rows: Vec<Row> = attendance_record::Entity::find()
            .select_only()
            .column(attendance_record::Column::SessionId)
            .column_as(Expr::expr(Func::count(Expr::col(attendance_record::Column::UserId))), "cnt")
            .filter(attendance_record::Column::UserId.in_subquery(student_ids_subq))
            .filter(attendance_record::Column::SessionId.is_in(session_ids.iter().cloned()))
            .group_by(attendance_record::Column::SessionId)
            .into_model::<Row>()
            .all(db)
            .await?;

        Ok(rows.into_iter().map(|r| (r.session_id, r.cnt)).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db;
    use chrono::{TimeZone, Utc};

    #[tokio::test]
    async fn test_session_code_rotates() {
        let db = setup_test_db().await;

        let s = Model::create(
            &db, 1, 1, "Lecture 5",
            true, 30,
            false, None, None,
            Some("00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"),
        ).await.unwrap();

        let t1 = Utc.with_ymd_and_hms(2025, 9, 8, 10, 0, 14).unwrap();
        let t2 = Utc.with_ymd_and_hms(2025, 9, 8, 10, 0, 31).unwrap();
        assert_ne!(s.current_code(t1), s.current_code(t2));
    }
}
