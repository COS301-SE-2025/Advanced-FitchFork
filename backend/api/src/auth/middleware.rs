use axum::{
    body::Body,
    extract::{ConnectInfo, FromRequestParts},
    http::{Method, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::TypedHeader;
use headers::{Origin, UserAgent};
use std::net::SocketAddr;
use tracing::info;
use crate::auth::claims::AuthUser;

/// Logs method, path, IP address, user ID (if authenticated), origin, and user-agent
/// for each incoming HTTP request. Automatically skips CORS preflight `OPTIONS` requests.
///
/// This middleware can help trace incoming traffic, debug authentication issues,
/// and monitor frontend usage patterns.
///
/// ### Usage:
/// Apply this middleware globally using:
///
/// ```ignore
/// use axum::Router;
/// use axum::middleware::from_fn;
/// use api::auth::middleware::log_request;
///
/// let app = Router::new().layer(from_fn(log_request));
/// ```
///
/// ### Fields Logged:
/// - `method`: HTTP method used (`GET`, `POST`, etc.)
/// - `path`: Requested URI path
/// - `ip`: Remote IP address of the client
/// - `user`: User ID if authenticated, `0` if not
/// - `origin`: Value of the `Origin` header if present
/// - `user_agent`: Value of the `User-Agent` header if present
pub async fn log_request(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let (mut parts, body) = req.into_parts();

    // Skip logging for preflight requests
    if parts.method == Method::OPTIONS {
        let req = Request::from_parts(parts, body);
        return Ok(next.run(req).await);
    }

    // Try extracting the user ID from claims
    let user_id = AuthUser::from_request_parts(&mut parts, &())
        .await
        .ok()
        .map(|AuthUser(c)| c.sub);

    // Try extracting Origin and User-Agent headers
    let origin = TypedHeader::<Origin>::from_request_parts(&mut parts, &())
        .await
        .ok()
        .map(|TypedHeader(o)| o.to_string());

    let user_agent = TypedHeader::<UserAgent>::from_request_parts(&mut parts, &())
        .await
        .ok()
        .map(|TypedHeader(ua)| ua.to_string());

    // Log relevant request metadata
    info!(
        method = ?parts.method,
        path = %parts.uri.path(),
        ip = %addr.ip(),
        user = user_id.unwrap_or(0),
        origin = origin.unwrap_or_else(|| "unknown".into()),
        user_agent = user_agent.unwrap_or_else(|| "unknown".into()),
        "Incoming request"
    );

    let req = Request::from_parts(parts, body);
    Ok(next.run(req).await)
}
