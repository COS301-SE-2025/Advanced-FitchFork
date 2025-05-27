use axum::{
    extract::Path,
    http::{Request, StatusCode},
    middleware::Next,
    body::Body,
    response::Response,
    Json,
};
use crate::auth::claims::AuthUser;
use crate::auth::extractors::{extract_module_id, PathParams};
use crate::response::ApiResponse;
use db::models::user::User;
use db::pool::get as get_pool;
use axum::extract::FromRequestParts;

/// A dummy struct used for responses that do not carry a data payload.
#[derive(serde::Serialize, Default)]
pub struct Empty;

/// Middleware to require authentication.
///
/// Verifies that the request includes a valid Bearer token and injects the
/// authenticated `AuthUser` into the request's extensions for downstream access.
///
/// # Returns
/// - `401 Unauthorized` if no valid token is provided.
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

/// Middleware to require admin privileges.
///
/// Ensures the authenticated user has `admin` set to true.
/// Fails with `403 Forbidden` if user is authenticated but not an admin.
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

/// Middleware to require lecturer access to a module.
///
/// Accepts a route path containing the module ID. Checks if the user is either
/// an admin or a lecturer in the module.
///
/// # Returns
/// - `403 Forbidden` if not a lecturer/admin.
/// - `400 Bad Request` if the path parameters are malformed.
pub async fn require_lecturer<T: Into<PathParams>>(
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
            Json(ApiResponse::error("Authentication required")),
        ))?
        .0
        .clone();

    if user.admin {
        return Ok(next.run(req).await);
    }

    let pool = get_pool();
    let is_lecturer = User::is_lecturer_in(Some(&pool), user.sub, module_id).await.unwrap_or(false);

    if is_lecturer {
        Ok(next.run(req).await)
    } else {
        Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Lecturer access required for this module")),
        ))
    }
}

/// Middleware to require tutor access to a module.
///
/// Works similarly to `require_lecturer` but checks tutor role.
/// Admins are automatically granted access.
pub async fn require_tutor<T: Into<PathParams>>(
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
            Json(ApiResponse::error("Authentication required")),
        ))?
        .0
        .clone();

    if user.admin {
        return Ok(next.run(req).await);
    }

    let pool = get_pool();
    let is_tutor = User::is_tutor_in(Some(&pool), user.sub, module_id).await.unwrap_or(false);

    if is_tutor {
        Ok(next.run(req).await)
    } else {
        Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Tutor access required for this module")),
        ))
    }
}

/// Middleware to require student access to a module.
///
/// Grants access to students or admins. Extracts the module ID from route.
pub async fn require_student<T: Into<PathParams>>(
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
            Json(ApiResponse::error("Authentication required")),
        ))?
        .0
        .clone();

    if user.admin {
        return Ok(next.run(req).await);
    }

    let pool = get_pool();
    let is_student = User::is_student_in(Some(&pool), user.sub, module_id).await.unwrap_or(false);

    if is_student {
        Ok(next.run(req).await)
    } else {
        Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Student access required for this module")),
        ))
    }
}

/// Middleware to require assignment to the module in any role (lecturer, tutor, student).
///
/// Ensures the user has at least one role in the given module. Admins bypass the check.
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
            Json(ApiResponse::error("Authentication required")),
        ))?
        .0
        .clone();

    if user.admin {
        return Ok(next.run(req).await);
    }

    let pool = get_pool();
    let is_lecturer = User::is_lecturer_in(Some(&pool), user.sub, module_id).await.unwrap_or(false);
    let is_tutor = User::is_tutor_in(Some(&pool), user.sub, module_id).await.unwrap_or(false);
    let is_student = User::is_student_in(Some(&pool), user.sub, module_id).await.unwrap_or(false);

    if is_lecturer || is_tutor || is_student {
        Ok(next.run(req).await)
    } else {
        Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Access denied: Not assigned to this module")),
        ))
    }
}
