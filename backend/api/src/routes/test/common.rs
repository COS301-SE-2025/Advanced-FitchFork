use services::user::User;
use validator::Validate;

/// Request body used by POST `/api/test/users` to create-or-update a user.
///
/// Notes:
/// - `admin` defaults to `false` if omitted.
/// - Plaintext `password` is accepted here only because this is a *test* helper.
///   Do not mirror this in production APIs.
#[derive(Debug, serde::Deserialize, Validate)]
pub struct UpsertUserRequest {
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
    pub admin: Option<bool>,
}

/// Minimal response payload returned from the test endpoints.
#[derive(Debug, serde::Serialize)]
pub struct TestUserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub admin: bool,
}

impl From<User> for TestUserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            email: u.email,
            admin: u.admin,
        }
    }
}
