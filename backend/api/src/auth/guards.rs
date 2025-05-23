use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    body::Body,
    response::Response,
    extract::FromRequestParts,
    Json,
};
use crate::auth::claims::AuthUser;
use crate::response::ApiResponse;

/// A dummy type for responses that don’t carry data
#[derive(serde::Serialize, Default)]
pub struct Empty;

/// Middleware to ensure the user is authenticated
pub async fn require_authenticated(
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    let (mut parts, body) = req.into_parts();

    let user = AuthUser::from_request_parts(&mut parts, &())
        .await
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::error("Authentication required")),
            )
        })?;

    let mut req = Request::from_parts(parts, body);
    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}

/// Middleware to ensure the user is authenticated *and* is an admin
pub async fn require_admin(
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    let (mut parts, body) = req.into_parts();

    let user = AuthUser::from_request_parts(&mut parts, &())
        .await
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::error("Authentication required")),
            )
        })?;

    if !user.0.admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Admin access required")),
        ));
    }

    let mut req = Request::from_parts(parts, body);
    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}