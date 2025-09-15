use serde::{Deserialize, Serialize};
use validator::Validate;
use services::user::User;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 1))]
    pub username: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct BulkCreateUsersRequest {
    #[validate(length(min = 1))]
    #[validate(nested)]
    pub users: Vec<CreateUserRequest>,
}

#[derive(Debug, Serialize)]
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
