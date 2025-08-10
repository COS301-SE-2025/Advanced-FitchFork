use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AnnouncementRequest {
    pub title: String,
    pub body: String,
    pub pinned: bool,
}
