use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64,
    pub exp: usize,
    pub admin: bool,
}

#[derive(Debug, Clone)]
pub struct AuthUser(pub Claims);
