use axum::{extract::{Path, State, FromRequestParts}, http::{Request, StatusCode}, middleware::Next, body::Body, response::{Response, IntoResponse}, Json};
use crate::auth::claims::AuthUser;
use crate::response::ApiResponse;
use db::models::user;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use sea_orm::EntityTrait;

// --- Role Based Access Guards ---

#[derive(serde::Serialize, Default)]
pub struct Empty;

/// Helper to extract and validate user from request extensions
fn get_authenticated_user(req: &Request<Body>) -> Result<&AuthUser, (StatusCode, Json<ApiResponse<Empty>>)> {
    req.extensions()
        .get::<AuthUser>()
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error("Authentication required"))
        ))
}

/// Helper to check if user has any of the specified roles
async fn user_has_any_role(
    db: &DatabaseConnection,
    user_id: i64,
    module_id: i64,
    roles: &[&str],
) -> bool {
    for role in roles {
        if user::Model::is_in_role(db, user_id, module_id, role).await.unwrap_or(false) {
            return true;
        }
    }
    false
}

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
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    let user = get_authenticated_user(&req)?;

    if !user.0.admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Admin access required"))
        ));
    }

    Ok(next.run(req).await)
}

/// Base role-based access guard that other guards can build upon
async fn require_role_base(
    State(db): State<DatabaseConnection>,
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next,
    required_roles: &[&str],
    failure_msg: &str,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    let user = get_authenticated_user(&req)?;
    let module_id = params.get("module_id")
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Missing or invalid module_id"))
        ))?;

    if user.0.admin {
        return Ok(next.run(req).await);
    }

    if user_has_any_role(&db, user.0.sub, module_id, required_roles).await {
        Ok(next.run(req).await)
    } else {
        Err((StatusCode::FORBIDDEN, Json(ApiResponse::error(failure_msg))))
    }
}

/// Guard for requiring lecturer access.
pub async fn require_lecturer(
    State(db): State<DatabaseConnection>,
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    require_role_base(
        State(db),
        Path(params),
        req,
        next,
        &["lecturer"],
        "Lecturer access required for this module"
    ).await
}

/// Guard for requiring tutor access.
pub async fn require_tutor(
    State(db): State<DatabaseConnection>,
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    require_role_base(
        State(db),
        Path(params),
        req,
        next,
        &["tutor"],
        "Tutor access required for this module"
    ).await
}

/// Guard for requiring student access.
pub async fn require_student(
    State(db): State<DatabaseConnection>,
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    require_role_base(
        State(db),
        Path(params),
        req,
        next,
        &["student"],
        "Student access required for this module"
    ).await
}

/// Guard for requiring lecturer or tutor access.
pub async fn require_lecturer_or_tutor(
    State(db): State<DatabaseConnection>,
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    require_role_base(
        State(db),
        Path(params),
        req,
        next,
        &["lecturer", "tutor"],
        "Lecturer or tutor access required for this module"
    ).await
}

/// Guard for requiring any assigned role (lecturer, tutor, student).
pub async fn require_assigned_to_module(
    State(db): State<DatabaseConnection>,
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    require_role_base(
        State(db),
        Path(params),
        req,
        next,
        &["lecturer", "tutor", "student"],
        "Access denied: Not assigned to this module"
    ).await
}

// --- Path ID Guards ---

async fn check_module_exists(id: i32, db: &DatabaseConnection) -> Result<(), StatusCode> {
    let found = db::models::module::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if found.is_some() {
        Ok(())
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn check_assignment_exists(id: i32, db: &DatabaseConnection) -> Result<(), StatusCode> {
    let found = db::models::assignment::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if found.is_some() {
        Ok(())
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn validate_known_ids(
    State(db): State<DatabaseConnection>,
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response, Response> {
    for (key, raw_val) in params {
        let id: i32 = raw_val.parse().map_err(|_| {
            StatusCode::BAD_REQUEST.into_response()
        })?;

        let res: Result<(), StatusCode> = match key.as_str() {
            "module_id"     => check_module_exists(id, &db).await,
            "assignment_id" => check_assignment_exists(id, &db).await,
            other => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("unknown path parameter `{}`", other),
                )
                    .into_response());
            }
        };

        if let Err(status) = res {
            return Err(status.into_response());
        }
    }

    Ok(next.run(req).await)
}