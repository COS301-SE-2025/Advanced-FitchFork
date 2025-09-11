use sea_orm::entity::prelude::*;
use sea_orm::EntityTrait;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "modules")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub code: String,
    pub year: i64,
    pub description: Option<String>,
    pub credits: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        has_many = "super::user_module_role::Entity",
        from = "Column::Id",
        to = "super::user_module_role::Column::ModuleId"
    )]
    UserModuleRole,
}

impl Related<super::user_module_role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserModuleRole.def()
    }

    fn via() -> Option<RelationDef> {
        None
    }
}

impl ActiveModelBehavior for ActiveModel {}