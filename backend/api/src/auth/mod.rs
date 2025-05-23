pub mod middleware;
pub mod claims;
pub mod extractors;
pub mod guards;

pub use claims::{Claims, AuthUser};
pub use guards::{require_authenticated, require_admin};
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Utc, Duration};
use common::config::Config;

pub fn generate_jwt(user_id: i64, admin: bool) -> (String, String) {
    let config = Config::get();

    let expiry = Utc::now() + Duration::minutes(config.jwt_duration_minutes as i64);
    let exp_timestamp = expiry.timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        admin,
        exp: exp_timestamp,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .expect("Token encoding failed");

    (token, expiry.to_rfc3339())
}