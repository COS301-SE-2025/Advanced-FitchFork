use axum::{
    extract::{Path, FromRequestParts},
    http::{Request, StatusCode},
    middleware::Next,
    body::Body,
    response::Response,
    Json,
};
use crate::auth::claims::AuthUser;
use crate::auth::extractors::{extract_module_id, PathParams};
use crate::response::ApiResponse;

use db::models::user;
use db::connect;

use futures::future::join_all;

#[derive(serde::Serialize, Default)]
pub struct Empty;

/// Basic guard to ensure the request is authenticated.
pub async fn require_authenticated(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    let (mut parts, body) = req.into_parts();

    let user = AuthUser::from_request_parts(&mut parts, &())
        .await
        .map_err(|_| (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error("Authentication required"))
        ))?;

    req = Request::from_parts(parts, body);
    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}

/// Admin-only guard.
pub async fn require_admin(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    let (mut parts, body) = req.into_parts();

    let user = AuthUser::from_request_parts(&mut parts, &())
        .await
        .map_err(|_| (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error("Authentication required"))
        ))?;

    if !user.0.admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Admin access required"))
        ));
    }

    req = Request::from_parts(parts, body);
    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}

/// Role-based access guard factory.
async fn require_role<T: Into<PathParams>>(
    Path(params): Path<T>,
    req: Request<Body>,
    next: Next,
    required_role: &str,
    failure_msg: &str,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    let Some(module_id) = extract_module_id(params) else {
        return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error("Invalid module path"))));
    };

    let user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error("Authentication required"))
        ))?
        .0
        .clone();

    if user.admin {
        return Ok(next.run(req).await);
    }

    let db = connect().await;
    let is_in_role = user::Model::is_in_role(&db, user.sub, module_id, required_role)
        .await
        .unwrap_or(false);

    if is_in_role {
        Ok(next.run(req).await)
    } else {
        Err((StatusCode::FORBIDDEN, Json(ApiResponse::error(failure_msg))))
    }
}

/// Guard for requiring lecturer access.
pub async fn require_lecturer<T: Into<PathParams>>(
    path: Path<T>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    require_role(path, req, next, "lecturer", "Lecturer access required for this module").await
}

/// Guard for requiring tutor access.
pub async fn require_tutor<T: Into<PathParams>>(
    path: Path<T>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    require_role(path, req, next, "tutor", "Tutor access required for this module").await
}

/// Guard for requiring student access.
pub async fn require_student<T: Into<PathParams>>(
    path: Path<T>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    require_role(path, req, next, "student", "Student access required for this module").await
}

/// Guard for requiring any assigned role (lecturer, tutor, student).
pub async fn require_assigned_to_module<T: Into<PathParams>>(
    Path(params): Path<T>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    let Some(module_id) = extract_module_id(params) else {
        return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::error("Invalid module path"))));
    };

    let user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error("Authentication required"))
        ))?
        .0
        .clone();

    if user.admin {
        return Ok(next.run(req).await);
    }

    let db = connect().await;
    let roles = ["lecturer", "tutor", "student"];
    let has_any = join_all(
        roles.iter().map(|r| {
            user::Model::is_in_role(&db, user.sub, module_id, r)
        })
    )
    .await
    .into_iter()
    .any(|res| res.unwrap_or(false));

    if has_any {
        Ok(next.run(req).await)
    } else {
        Err((StatusCode::FORBIDDEN, Json(ApiResponse::error("Access denied: Not assigned to this module"))))
    }
}
