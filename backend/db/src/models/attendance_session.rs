use chrono::{DateTime, Utc};
use sea_orm::EntityTrait;
use sea_orm::entity::prelude::*;

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
    #[sea_orm(
        belongs_to = "super::module::Entity",
        from = "Column::ModuleId",
        to = "super::module::Column::Id"
    )]
    Module,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::CreatedBy",
        to = "super::user::Column::Id"
    )]
    Creator,
    #[sea_orm(has_many = "super::attendance_record::Entity")]
    Records,
}

impl Related<super::module::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Module.def()
    }
    fn via() -> Option<RelationDef> {
        None
    }
}

impl Related<super::attendance_record::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Records.def()
    }
    fn via() -> Option<RelationDef> {
        None
    }
}

impl ActiveModelBehavior for ActiveModel {}
