use db::models::tickets::Model as TicketModel;
use db::models::user_module_role::Role;
use db::models::{user, user_module_role};
use sea_orm::entity::prelude::*;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QuerySelect};
use serde::{Deserialize, Serialize};

pub async fn is_valid(module_id: i64, user_id: i64, ticket_id: i64, db: &DatabaseConnection) -> bool {
    let is_student = user_module_role::Entity::find()
        .filter(user_module_role::Column::UserId.eq(user_id))
        .filter(user_module_role::Column::ModuleId.eq(module_id))
        .filter(user_module_role::Column::Role.eq(Role::Student))
        .join(JoinType::InnerJoin, user_module_role::Relation::User.def())
        .filter(user::Column::Admin.eq(false))
        .one(db)
        .await
        .map(|opt| opt.is_some())
        .unwrap_or(false);

    let is_author = TicketModel::is_author(ticket_id, user_id, db).await;

    !is_student && is_author
}
 
 #[derive(Debug, Serialize, Deserialize)]
pub struct TicketResponse {
    pub id: i64,
    pub assignment_id: i64,
    pub user_id: i64,
    pub title: String,
    pub description: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}
impl From<TicketModel> for TicketResponse {
    fn from(ticket: TicketModel) -> Self {
        Self {
            id: ticket.id,
            assignment_id: ticket.assignment_id,
            user_id: ticket.user_id,
            title: ticket.title,
            description: ticket.description,
            status: ticket.status.to_string(),
            created_at: ticket.created_at.to_rfc3339(),
            updated_at: ticket.updated_at.to_rfc3339(),
        }
    }
}