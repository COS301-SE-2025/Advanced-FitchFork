use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize)]
#[sea_orm(table_name = "attendance_records")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub session_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i64,

    pub taken_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub token_window: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::attendance_session::Entity",
        from = "Column::SessionId",
        to = "super::attendance_session::Column::Id"
    )]
    Session,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::attendance_session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Session.def()
    }
    fn via() -> Option<RelationDef> {
        None
    }
}
impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
    fn via() -> Option<RelationDef> {
        None
    }
}

impl ActiveModelBehavior for ActiveModel {}
