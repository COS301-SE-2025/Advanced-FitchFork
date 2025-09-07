use crate::service::{Service, ToActiveModel};
use db::{
    models::password_reset_token::{ActiveModel, Entity, Model},
    repositories::{password_reset_token_repository::PasswordResetTokenRepository, repository::Repository},
    comparisons::Comparison,
    filters::PasswordResetTokenFilter,
};
use sea_orm::{DbErr, Set};
use chrono::{Utc, Duration};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

#[derive(Debug, Clone)]
pub struct CreatePasswordResetToken {
    pub user_id: i64,
    pub expiry_minutes: i64
}

#[derive(Debug, Clone)]
pub struct UpdatePasswordResetToken {
    pub id: i64,
    pub used: Option<bool>,
}

impl ToActiveModel<Entity> for CreatePasswordResetToken {
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
        let token = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect::<String>();
        
        let now = Utc::now();
        Ok(ActiveModel {
            user_id: Set(self.user_id),
            token: Set(token),
            expires_at: Set(now + Duration::minutes(self.expiry_minutes)),
            used: Set(false),
            created_at: Set(now),
            ..Default::default()
        })
    }
}

impl ToActiveModel<Entity> for UpdatePasswordResetToken {
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
        let token = match PasswordResetTokenRepository::find_by_id(self.id).await {
            Ok(Some(token)) => token,
            Ok(None) => {
                return Err(DbErr::RecordNotFound(format!("Token not found for ID {}", self.id)));
            }
            Err(err) => return Err(err),
        };
        let mut active: ActiveModel = token.into();

        if let Some(used) = self.used {
            active.used = Set(used);
        }

        Ok(active)
    }
}

pub struct PasswordResetTokenService;

impl<'a> Service<'a, Entity, CreatePasswordResetToken, UpdatePasswordResetToken, PasswordResetTokenFilter, PasswordResetTokenRepository> for PasswordResetTokenService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓
}

impl PasswordResetTokenService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓

    pub async fn find_valid_token(
        token: String,
    ) -> Result<Option<Model>, DbErr> {
        PasswordResetTokenRepository::find_one(
            PasswordResetTokenFilter {
                token: Some(Comparison::eq(token)),
                used: Some(Comparison::eq(false)),
                expires_at: Some(Comparison::gt(Utc::now())),
                ..Default::default()
            },
            None,
        ).await
    }
}