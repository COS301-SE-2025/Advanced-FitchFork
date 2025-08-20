
use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "announcements")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    pub module_id: i64,
    pub user_id: i64,

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
        module_id: i64,
        user_id: i64,
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

    pub async fn delete(db: &DbConn, id: i64) -> Result<(), DbErr> {
        Entity::delete_by_id(id).exec(db).await?;
        Ok(())
    }

    pub async fn update(
        db: &DbConn,
        id: i64,
        title: &str,
        body: &str,
        pinned: bool,
    ) -> Result<Model, DbErr> {
        let mut announcement = ActiveModel {
            id: Set(id),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        if !title.is_empty() {
            announcement.title = Set(title.to_owned());
        }
        if !body.is_empty() {
            announcement.body = Set(body.to_owned());
        }
        announcement.pinned = Set(pinned);

        announcement.update(db).await
    }
}
