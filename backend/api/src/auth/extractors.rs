use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use axum_extra::extract::TypedHeader;
use headers::{Authorization, authorization::Bearer};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use std::env;

use crate::auth::claims::{Claims, AuthUser};

/// Implements extraction of `AuthUser` from request headers.
///
/// This middleware checks for a valid Bearer token in the `Authorization` header,
/// verifies the JWT using the secret from `JWT_SECRET` environment variable,
/// and extracts the user claims into an `AuthUser` instance.
///
/// # Errors
/// - Returns `401 Unauthorized` if the header is missing, malformed, or the token is invalid or expired.
///
/// # Example
/// ```rust
/// async fn protected_route(user: AuthUser) -> impl IntoResponse {
///     // User is now available
/// }
/// ```
#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, _state)
                .await
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing or invalid Authorization header"))?;

        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(env::var("JWT_SECRET").unwrap().as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired token"))?;

        Ok(AuthUser(token_data.claims))
    }
}

/// Extracts the `module_id` from common Axum route parameter patterns.
///
/// Useful when matching on routes with 1–3 integer path parameters,
/// e.g. `/modules/:module_id`, `/modules/:module_id/assignments/:assignment_id`, etc.
///
/// # Returns
/// - `Some(module_id)` if a valid first parameter is found.
/// - `None` if the tuple structure is invalid or missing.
///
/// # Example
/// ```rust
/// let id = extract_module_id((42, 7)); // returns Some(42)
/// ```
pub fn extract_module_id<T: Into<PathParams>>(params: T) -> Option<i64> {
    let v: Vec<i64> = match params.into() {
        PathParams::One(a) => vec![a],
        PathParams::Two(a, _) => vec![a],
        PathParams::Three(a, _, _) => vec![a],
        PathParams::Invalid => vec![],
    };
    v.first().copied()
}

/// Enum to generalize Axum path parameter patterns for 1–3 integers.
///
/// Used internally to simplify extraction logic for module-related IDs.
#[derive(Debug)]
pub enum PathParams {
    One(i64),
    Two(i64, i64),
    Three(i64, i64, i64),
    Invalid,
}

// Conversion implementations for Axum-style tuples into `PathParams`
impl From<()> for PathParams {
    fn from(_: ()) -> Self {
        PathParams::Invalid
    }
}

impl From<(i64,)> for PathParams {
    fn from(p: (i64,)) -> Self {
        PathParams::One(p.0)
    }
}

impl From<(i64, i64)> for PathParams {
    fn from(p: (i64, i64)) -> Self {
        PathParams::Two(p.0, p.1)
    }
}

impl From<(i64, i64, i64)> for PathParams {
    fn from(p: (i64, i64, i64)) -> Self {
        PathParams::Three(p.0, p.1, p.2)
    }
}
