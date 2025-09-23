//! Ticket utilities.
//!
//! This module provides helper functions and types for ticket-related endpoints.
//!
//! It includes:
//! - `is_valid`: checks whether a user is authorized to access or modify a ticket.
//! - `TicketResponse`: a serializable response type for ticket API endpoints.

use db::models::user::Model as UserModel;
use db::models::{
    UserModuleRole as Entity,
    tickets::Model as TicketModel,
    user_module_role::{Column, Role},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

/// Returns whether `user_id` is allowed to view/post on a ticket in `module_id`.
///
/// Admins are always allowed.
///
/// Rules:
/// - `is_admin == true` → allowed
/// - Ticket **author** → allowed
/// - Module **staff** (Lecturer, AssistantLecturer, Tutor) → allowed
pub async fn is_valid(
    user_id: i64,
    ticket_id: i64,
    module_id: i64,
    is_admin: bool,
    db: &DatabaseConnection,
) -> bool {
    // Admin override
    if is_admin {
        return true;
    }

    // Author of the ticket?
    if TicketModel::is_author(ticket_id, user_id, db).await {
        return true;
    }

    // Staff on this module?
    let staff_roles = [Role::Lecturer, Role::AssistantLecturer, Role::Tutor];
    Entity::find()
        .filter(Column::UserId.eq(user_id))
        .filter(Column::ModuleId.eq(module_id))
        .filter(Column::Role.is_in(staff_roles))
        .one(db)
        .await
        .unwrap_or(None)
        .is_some()
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
