use serde::Serialize;
use serde::Deserialize;
use services::user::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserModule {
    pub id: i64,
    pub code: String,
    pub year: i32,
    pub description: String,
    pub credits: i64,
    pub role: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub admin: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            admin: user.admin,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        }
    }
}