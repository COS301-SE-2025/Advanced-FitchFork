use jsonwebtoken::{EncodingKey, DecodingKey};
use sea_orm::DatabaseConnection;
use std::env;

// A central place to store the database, environment variables and other stuff so that it only has to be loaded once etc.
// This is not being used yet. Work in progress.

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    jwt_encoding_key: EncodingKey,
    jwt_decoding_key: DecodingKey,
    jwt_duration_minutes: i64,
}

impl AppState {
    pub fn new(db: DatabaseConnection) -> Self {
        let secret = env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set at startup");
        
        let jwt_duration_minutes = env::var("JWT_DURATION_MINUTES")
            .expect("JWT_DURATION_MINUTES must be set at startup")
            .parse::<i64>()
            .expect("JWT_DURATION_MINUTES must be a valid integer");

        Self {
            db,
            jwt_encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            jwt_decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            jwt_duration_minutes,
        }
    }

    pub fn encoding_key(&self) -> &EncodingKey {
        &self.jwt_encoding_key
    }

    pub fn decoding_key(&self) -> &DecodingKey {
        &self.jwt_decoding_key
    }

    pub fn jwt_duration_minutes(&self) -> i64 {
        self.jwt_duration_minutes
    }
}