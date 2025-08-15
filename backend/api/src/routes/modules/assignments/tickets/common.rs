use db::models::{tickets::Model as TicketModel, user_module_role::{Column, Role}, UserModuleRole as Entity};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use serde::{Deserialize, Serialize};

pub async fn is_valid(
    user_id: i64,
    ticket_id: i64,
    module_id: i64,
    db: &DatabaseConnection,
) -> bool {
    let is_author = TicketModel::is_author(ticket_id, user_id, db).await;
    let staff_roles = vec![Role::Lecturer, Role::AssistantLecturer, Role::Tutor];
    let is_staff = Entity::find()
        .filter(Column::UserId.eq(user_id))
        .filter(Column::ModuleId.eq(module_id))
        .filter(Column::Role.is_in(staff_roles))
        .one(db)
        .await
        .unwrap_or(None)
        .is_some();

    is_author || is_staff
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
