use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::{Set, NotSet};
use sea_orm::IntoActiveModel;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use crate::get_connection;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "password_reset_tokens")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn new(user_id: i64, expiry_minutes: i64) -> Self {
        let token = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect::<String>();

        Self {
            id: 0,
            user_id,
            token,
            expires_at: Utc::now() + Duration::minutes(expiry_minutes),
            used: false,
            created_at: Utc::now(),
        }
    }

    pub async fn create(
        user_id: i64,
        expiry_minutes: i64,
    ) -> Result<Self, DbErr> {
        let model = Self::new(user_id, expiry_minutes);
        let mut active_model = model.into_active_model();
        active_model.id = NotSet;
        let db = get_connection().await;
        active_model.insert(db).await
    }

    pub async fn find_valid_token(
        token: &str,
    ) -> Result<Option<Self>, DbErr> {
        let db = get_connection().await;
        Entity::find()
            .filter(Column::Token.eq(token))
            .filter(Column::Used.eq(false))
            .filter(Column::ExpiresAt.gt(Utc::now()))
            .one(db)
            .await
    }

    pub async fn mark_as_used(&self) -> Result<(), DbErr> {
        let mut active_model: ActiveModel = self.clone().into();
        active_model.used = Set(true);
        let db = get_connection().await;
        active_model.update(db).await?;
        Ok(())
    }
} 