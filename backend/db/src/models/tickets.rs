use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use sea_orm::DeriveActiveEnum;
use sea_orm::QueryFilter;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "tickets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    pub assignment_id: i64,
    pub user_id: i64,

    pub title: String,
    pub description: String,

    pub status: TicketStatus,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(
    Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Display, EnumString, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "ticket_status")]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum TicketStatus {
    #[sea_orm(string_value = "open")]
    Open,

    #[sea_orm(string_value = "closed")]
    Closed,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::assignment::Entity",
        from = "Column::AssignmentId",
        to = "super::assignment::Column::Id"
    )]
    Assignment,

    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::assignment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Assignment.def()
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
        assignment_id: i64,
        user_id: i64,
        title: &str,
        description: &str,
    ) -> Result<Model, DbErr> {
        let now = Utc::now();

        let active_model = ActiveModel {
            assignment_id: Set(assignment_id),
            user_id: Set(user_id),
            title: Set(title.to_owned()),
            description: Set(description.to_owned()),
            status: Set(TicketStatus::Open),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        active_model.insert(db).await
    }

    pub async fn find_by_user_and_assignment(
        db: &DbConn,
        user_id: i64,
        assignment_id: i64,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::AssignmentId.eq(assignment_id))
            .one(db)
            .await
    }

    pub async fn set_open(db: &DbConn, ticket_id: i64) -> Result<Model, DbErr> {
        let model = Entity::find_by_id(ticket_id).one(db).await?;

        let model = match model {
            Some(m) => m,
            None => return Err(DbErr::RecordNotFound("Ticket not found".to_string())),
        };

        let mut active_model: ActiveModel = model.into();

        active_model.status = Set(TicketStatus::Open);
        active_model.updated_at = Set(Utc::now());
        active_model.update(db).await
    }

    pub async fn set_closed(db: &DbConn, ticket_id: i64) -> Result<Model, DbErr> {
        let model = Entity::find_by_id(ticket_id).one(db).await?;

        let model = match model {
            Some(m) => m,
            None => return Err(DbErr::RecordNotFound("Ticket not found".to_string())),
        };

        let mut active_model: ActiveModel = model.into();

        active_model.status = Set(TicketStatus::Closed);
        active_model.updated_at = Set(Utc::now());
        active_model.update(db).await
    }

    pub async fn find_all_for_assignment(
        db: &DbConn,
        assignment_id: i64,
        user_id: i64,
        user_is_admin: bool,
    ) -> Result<Vec<Model>, DbErr> {
        let mut query = Entity::find().filter(Column::AssignmentId.eq(assignment_id));

        if !user_is_admin {
            query = query.filter(Column::UserId.eq(user_id));
        }

        query.all(db).await
    }

    pub async fn is_author(ticket_id: i64, user_id: i64, db: &DbConn) -> bool {
        let ticket = Entity::find_by_id(ticket_id).one(db).await;
        match ticket {
            Ok(Some(t)) => t.user_id == user_id,
            _ => false,
        }
    }
}
