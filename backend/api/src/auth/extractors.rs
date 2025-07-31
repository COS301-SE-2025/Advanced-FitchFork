use axum::{
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
/// ```ignore
/// use axum::response::IntoResponse;
/// use api::auth::claims::AuthUser;
///
/// async fn protected_route(user: AuthUser) -> impl IntoResponse {
///     // User is now available
///     format!("User ID: {}", user.0.sub)
/// }
/// ```
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, _state)
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