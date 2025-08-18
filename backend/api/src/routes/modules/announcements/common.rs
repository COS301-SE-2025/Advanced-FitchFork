//! # Announcement Request DTO
//!
//! Represents the payload for creating or updating an announcement.
//! Used in POST and PUT requests under the `/announcements` route group.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AnnouncementRequest {
    pub title: String,
    pub body: String,
    pub pinned: bool,
}
