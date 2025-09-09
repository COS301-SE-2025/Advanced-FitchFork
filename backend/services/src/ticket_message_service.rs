use crate::service::{Service, AppError, ToActiveModel};
use db::{
    models::ticket_messages::{ActiveModel, Entity},
    repositories::{ticket_message_repository::TicketMessageRepository, repository::Repository},
};
use sea_orm::{DbErr, Set};
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct CreateTicketMessage {
    pub ticket_id: i64,
    pub user_id: i64,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct UpdateTicketMessage {
    pub id: i64,
    pub content: Option<String>,
}

impl ToActiveModel<Entity> for CreateTicketMessage {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let now = Utc::now();
        Ok(ActiveModel {
            ticket_id: Set(self.ticket_id),
            user_id: Set(self.user_id),
            content: Set(self.content.to_owned()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
    }
}

impl ToActiveModel<Entity> for UpdateTicketMessage {
    async fn into_active_model(self) -> Result<ActiveModel, AppError> {
        let message = match TicketMessageRepository::find_by_id(self.id).await {
            Ok(Some(message)) => message,
            Ok(None) => {
                return Err(AppError::from(DbErr::RecordNotFound(format!("Message not found for ID {}", self.id))));
            }
            Err(err) => return Err(AppError::from(err)),
        };
        let mut active: ActiveModel = message.into();

        if let Some(content) = self.content {
            active.content = Set(content);
        }

        active.updated_at = Set(Utc::now());

        Ok(active)
    }
}

pub struct TicketMessageService;

impl<'a> Service<'a, Entity, CreateTicketMessage, UpdateTicketMessage, TicketMessageRepository> for TicketMessageService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓
}

impl TicketMessageService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓


    pub async fn is_author(
        message_id: i64,
        user_id: i64,
    ) -> bool {
        let message = TicketMessageRepository::find_by_id(message_id).await;
        match message {
            Ok(Some(t)) => t.user_id == user_id,
            _ => false,
        }
    }
}