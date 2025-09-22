use crate::ws::WebSocketManager;
use dotenvy::dotenv;
use jsonwebtoken::{DecodingKey, EncodingKey};
use std::env;
use std::sync::OnceLock;

static APP_STATE: OnceLock<AppState> = OnceLock::new();

#[derive(Clone)]
pub struct AppState {
    ws: WebSocketManager,
    jwt_encoding_key: EncodingKey,
    jwt_decoding_key: DecodingKey,
    jwt_duration_minutes: i64,
    test_mode: bool,
}

impl AppState {
    pub fn init(test_mode: bool) -> &'static Self {
        APP_STATE.get_or_init(|| {
            dotenv().ok();

            let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set at startup");

            let jwt_duration_minutes = env::var("JWT_DURATION_MINUTES")
                .expect("JWT_DURATION_MINUTES must be set at startup")
                .parse::<i64>()
                .expect("JWT_DURATION_MINUTES must be a valid integer");

            Self {
                ws: WebSocketManager::new(),
                jwt_encoding_key: EncodingKey::from_secret(secret.as_bytes()),
                jwt_decoding_key: DecodingKey::from_secret(secret.as_bytes()),
                jwt_duration_minutes,
                test_mode,
            }
        })
    }

    pub fn get() -> &'static Self {
        APP_STATE.get().expect("AppState not initialized")
    }

    pub fn ws(&self) -> &WebSocketManager {
        &self.ws
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

    pub fn is_test_mode(&self) -> bool {
        self.test_mode
    }
}
