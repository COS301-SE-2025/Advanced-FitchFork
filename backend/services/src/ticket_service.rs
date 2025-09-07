use crate::service::{Service, ToActiveModel};
use db::{
    models::tickets::{ActiveModel, Entity, TicketStatus},
    repositories::{ticket_repository::TicketRepository, repository::Repository},
    filters::TicketFilter,
};
use sea_orm::{DbErr, Set};
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct CreateTicket {
    pub assignment_id: i64,
    pub user_id: i64,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct UpdateTicket {
    pub id: i64,
    pub status: Option<TicketStatus>,
}

impl ToActiveModel<Entity> for CreateTicket {
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
        let now = Utc::now();
        Ok(ActiveModel {
            assignment_id: Set(self.assignment_id),
            user_id: Set(self.user_id),
            title: Set(self.title.to_owned()),
            description: Set(self.description.to_owned()),
            status: Set(TicketStatus::Open),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
    }
}

impl ToActiveModel<Entity> for UpdateTicket {
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
        let ticket = match TicketRepository::find_by_id(self.id).await {
            Ok(Some(ticket)) => ticket,
            Ok(None) => {
                return Err(DbErr::RecordNotFound(format!("Ticket not found for ID {}", self.id)));
            }
            Err(err) => return Err(err),
        };
        let mut active: ActiveModel = ticket.into();

        if let Some(status) = self.status {
            active.status = Set(status);
        }

        active.updated_at = Set(Utc::now());

        Ok(active)
    }
}

pub struct TicketService;

impl<'a> Service<'a, Entity, CreateTicket, UpdateTicket, TicketFilter, TicketRepository> for TicketService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓
}

impl TicketService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

    pub async fn is_author(
        ticket_id: i64,
        user_id: i64,
    ) -> bool {
        let ticket = TicketRepository::find_by_id(ticket_id).await;
        match ticket {
            Ok(Some(t)) => t.user_id == user_id,
            _ => false,
        }
    }
}