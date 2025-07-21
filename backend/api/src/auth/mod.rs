pub mod middleware;
pub mod claims;
pub mod extractors;
pub mod guards;

pub use claims::{Claims, AuthUser};

use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Utc, Duration};
use std::env;

/// Generates a JWT and its expiry timestamp for a given user.
pub fn generate_jwt(user_id: i64, admin: bool) -> (String, String) {
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let jwt_duration_minutes: i64 = env::var("JWT_DURATION_MINUTES")
        .expect("JWT_DURATION_MINUTES must be set")
        .parse()
        .expect("JWT_DURATION_MINUTES must be a valid integer");

    let expiry = Utc::now() + Duration::minutes(jwt_duration_minutes);
    let exp_timestamp = expiry.timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        admin,
        exp: exp_timestamp,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .expect("Token encoding failed");

    (token, expiry.to_rfc3339())
}