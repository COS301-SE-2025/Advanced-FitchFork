use sea_orm::{entity::prelude::*, ActiveValue::Set, QueryOrder};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "ticket_messages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    pub ticket_id: i64,
    pub user_id: i64,

    pub content: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tickets::Entity",
        from = "Column::TicketId",
        to = "super::tickets::Column::Id"
    )]
    Ticket,

    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::tickets::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ticket.def()
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
        ticket_id: i64,
        user_id: i64,
        content: &str,
    ) -> Result<Model, DbErr> {
        let now = Utc::now();

        let active = ActiveModel {
            ticket_id: Set(ticket_id),
            user_id: Set(user_id),
            content: Set(content.to_owned()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        active.insert(db).await
    }

    pub async fn find_all_for_ticket(
        db: &DbConn,
        ticket_id: i64,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::TicketId.eq(ticket_id))
            .order_by_asc(Column::CreatedAt)
            .all(db)
            .await
    }

    pub async fn delete(
        db: &DbConn,
        message_id: i64,
    ) -> Result<(), DbErr> {
        Entity::delete_by_id(message_id).exec(db).await?;
        Ok(())
    }

    pub async fn is_author(message_id: i64, user_id: i64, db: &DbConn) -> bool {
        let message = Entity::find_by_id(message_id).one(db).await;
        match message {
            Ok(Some(t)) => t.user_id == user_id,
            _ => false,
        }
    }
}
