use sea_orm::entity::prelude::*;
use sea_orm::{DeriveActiveEnum, ActiveValue};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use sea_orm::QueryFilter;

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

    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Display, EnumString, Serialize, Deserialize)]
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

impl ActiveModelBehavior for ActiveModel {
 
}

impl Model {
    pub async fn create(
        db: &DbConn,
        assignment_id: i64,
        user_id: i64,
        title: &str,
        description: &str,
    ) -> Result<Model, DbErr> {
        let now = chrono::Utc::now().naive_utc();

        let active_model = ActiveModel {
            assignment_id: ActiveValue::Set(assignment_id),
            user_id: ActiveValue::Set(user_id),
            title: ActiveValue::Set(title.to_owned()),
            description: ActiveValue::Set(description.to_owned()),
            status: ActiveValue::Set(TicketStatus::Open),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
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

    pub fn is_author(&self, user_id: i64) -> bool {
        self.user_id == user_id
    }
}