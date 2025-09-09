use crate::service::{Service, ToActiveModel};
use db::{
    models::announcements::{ActiveModel, Entity},
    repositories::{announcement_repository::AnnouncementRepository, repository::Repository},
};
use sea_orm::{DbErr, Set};
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct CreateAnnouncement {
    pub module_id: i64,
    pub user_id: i64,
    pub title: String,
    pub body: String,
    pub pinned: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateAnnouncement {
    pub id: i64,
    pub title: Option<String>,
    pub body: Option<String>,
    pub pinned: Option<bool>,
}

impl ToActiveModel<Entity> for CreateAnnouncement {
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
        let now = Utc::now();
        Ok(ActiveModel {
            module_id: Set(self.module_id),
            user_id: Set(self.user_id),
            title: Set(self.title.to_owned()),
            body: Set(self.body.to_owned()),
            pinned: Set(self.pinned),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
    }
}

impl ToActiveModel<Entity> for UpdateAnnouncement {
    async fn into_active_model(self) -> Result<ActiveModel, DbErr> {
        let announcement = match AnnouncementRepository::find_by_id(self.id).await {
            Ok(Some(announcement)) => announcement,
            Ok(None) => {
                return Err(DbErr::RecordNotFound(format!("Announcement not found for ID {}", self.id)));
            }
            Err(err) => return Err(err),
        };
        let mut active: ActiveModel = announcement.into();

        if let Some(title) = self.title {
            active.title = Set(title);
        }

        if let Some(body) = self.body {
            active.body = Set(body);
        }

        if let Some(pinned) = self.pinned {
            active.pinned = Set(pinned);
        }

        active.updated_at = Set(Utc::now());

        Ok(active)
    }
}

pub struct AnnouncementService;

impl<'a> Service<'a, Entity, CreateAnnouncement, UpdateAnnouncement, AnnouncementRepository> for AnnouncementService {
    // ↓↓↓ OVERRIDE DEFAULT BEHAVIOR IF NEEDED HERE ↓↓↓
}

impl AnnouncementService {
    // ↓↓↓ CUSTOM METHODS CAN BE DEFINED HERE ↓↓↓
}