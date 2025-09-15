use axum::{
    extract::{FromRequestParts},
    http::{request::Parts, StatusCode},
};
use axum_extra::extract::TypedHeader;
use headers::{Authorization, authorization::Bearer};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use std::collections::HashMap;
use crate::auth::claims::{Claims, AuthUser};
use util::config;

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

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Try Authorization header first
        if let Ok(TypedHeader(Authorization(bearer))) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await
        {
            return decode_token(bearer.token());
        }

        // Fallback to query param `?token=...`
        if let Some(query) = &parts.uri.query() {
            let parsed: HashMap<String, String> = url::form_urlencoded::parse(query.as_bytes())
                .into_owned()
                .collect();

            if let Some(token) = parsed.get("token") {
                return decode_token(token);
            }
        }

        Err((StatusCode::UNAUTHORIZED, "Missing or invalid Authorization header"))
    }
}

fn decode_token(token: &str) -> Result<AuthUser, (StatusCode, &'static str)> {
    let secret = config::jwt_secret();
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired token"))?;

    Ok(AuthUser(data.claims))
}