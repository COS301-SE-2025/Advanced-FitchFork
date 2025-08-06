use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "announcements")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    pub module_id: i32,
    pub user_id: i32,

    pub title: String,
    pub body: String,
    pub pinned: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::module::Entity",
        from = "Column::ModuleId",
        to = "super::module::Column::Id",
        on_delete = "Cascade"
    )]
    Module,

    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::module::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Module.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {

    pub async fn create(
        db: &DbConn,
        module_id: i32,
        user_id: i32,
        title: &str,
        body: &str,
        pinned: bool,
    ) -> Result<Model, DbErr> {
        let now = Utc::now();
        let announcement = ActiveModel {
            module_id: Set(module_id),
            user_id: Set(user_id),
            title: Set(title.to_owned()),
            body: Set(body.to_owned()),
            pinned: Set(pinned),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        announcement.insert(db).await
    }

    pub async fn find_by_user_id(db: &DbConn, user_id: i64) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .all(db)
            .await
    }

    pub async fn find_by_module_id(db: &DbConn, module_id: i32) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::ModuleId.eq(module_id))
            .all(db)
            .await
    }
}
