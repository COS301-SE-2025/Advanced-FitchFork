use chrono::{DateTime, Duration, Utc};
use rand::distributions::Alphanumeric;
use rand::{Rng, thread_rng};
use sea_orm::ActiveValue::{NotSet, Set};
use sea_orm::IntoActiveModel;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

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
        db: &DatabaseConnection,
        user_id: i64,
        expiry_minutes: i64,
    ) -> Result<Self, DbErr> {
        let model = Self::new(user_id, expiry_minutes);
        let mut active_model = model.into_active_model();
        active_model.id = NotSet;
        active_model.insert(db).await
    }

    pub async fn find_valid_token(
        db: &DatabaseConnection,
        token: &str,
    ) -> Result<Option<Self>, DbErr> {
        Entity::find()
            .filter(Column::Token.eq(token))
            .filter(Column::Used.eq(false))
            .filter(Column::ExpiresAt.gt(Utc::now()))
            .one(db)
            .await
    }

    pub async fn mark_as_used(&self, db: &DatabaseConnection) -> Result<(), DbErr> {
        let mut active_model: ActiveModel = self.clone().into();
        active_model.used = Set(true);
        active_model.update(db).await?;
        Ok(())
    }
}
