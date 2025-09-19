//! Authentication utilities and JWT helpers.
//!
//! Provides claims, guards, extractors, middleware, and a function to generate JWTs.

pub mod claims;
pub mod extractors;
pub mod guards;
pub mod middleware;

pub use claims::{AuthUser, Claims};

use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use util::config;

/// Generates a JWT and its expiry timestamp for a given user.
pub fn generate_jwt(user_id: i64, admin: bool) -> (String, String) {
    let jwt_secret = config::jwt_secret();
    let jwt_duration_minutes: i64 = config::jwt_duration_minutes() as i64;

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
