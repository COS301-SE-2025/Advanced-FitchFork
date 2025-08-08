use db::models::tickets::Model as TicketModel;
use db::models::user::Model as UserModel;
use db::models::assignment::{Entity as AssignmentEntity, Column as AssignmentColumn};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use serde::{Deserialize, Serialize};

pub async fn is_valid(user_id: i64, ticket_id: i64, db: &DatabaseConnection) -> bool {
    // If author, always allowed
    if TicketModel::is_author(ticket_id, user_id, db).await {
        return true;
    }

    // Get ticket
    let ticket = match TicketModel::get_by_id(db, ticket_id).await {
        Ok(Some(t)) => t,
        _ => return false,
    };

    // Get assignment to find module ID
    let assignment = match AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(ticket.assignment_id))
        .one(db)
        .await
    {
        Ok(Some(a)) => a,
        _ => return false,
    };

    // If the user is a student and not the author → deny
    if UserModel::is_in_role(db, user_id, assignment.module_id, "Student")
        .await
        .unwrap_or(false)
    {
        return false;
    }

    // All other roles (tutor, lecturer, etc.) → allow
    true
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

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
}

impl From<UserModel> for UserResponse {
    fn from(user: UserModel) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TicketWithUserResponse {
    pub ticket: TicketResponse,
    pub user: UserResponse,
}