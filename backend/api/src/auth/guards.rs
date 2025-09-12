use axum::{
    extract::{Path, FromRequestParts},
    http::{Request, StatusCode},
    middleware::Next,
    body::Body,
    response::{Response, IntoResponse},
    Json
};
use crate::auth::claims::AuthUser;
use crate::response::ApiResponse;
use std::collections::HashMap;
use util::filters::FilterParam;
use services::service::Service;
use services::user::UserService;
use services::user_module_role::UserModuleRoleService;
use services::assignment::AssignmentService;
use services::module::ModuleService;
use services::assignment_task::AssignmentTaskService;
use services::assignment_submission::AssignmentSubmissionService;
use services::assignment_file::AssignmentFileService;
use services::ticket::TicketService;
use services::ticket_message::TicketMessageService;
use services::announcement::AnnouncementService;
use services::plagiarism_case::PlagiarismCaseService;

// --- Role Based Access Guards ---

#[derive(serde::Serialize, Default)]
pub struct Empty;

/// Helper to extract, validate user from request extensions and insert the back into the request
async fn extract_and_insert_authuser(
    mut req: Request<Body>
) -> Result<(Request<Body>, AuthUser), (StatusCode, Json<ApiResponse<Empty>>)> {
    let (mut parts, body) = req.into_parts();
    let user = AuthUser::from_request_parts(&mut parts, &())
        .await
        .map_err(|_| (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error("Authentication required"))
        ))?;
    
    req = Request::from_parts(parts, body);
    req.extensions_mut().insert(user.clone());
    Ok((req, user))
}

/// Helper to check if user has any of the specified roles
async fn user_has_any_role(
    user_id: i64,
    module_id: i64,
    roles: &[&str],
) -> bool {
    for role in roles {
        if UserModuleRoleService::is_in_role(user_id, module_id, role.to_string()).await.unwrap_or(false) {
            return true;
        }
    }
    false
}

/// Basic guard to ensure the request is authenticated.
pub async fn require_authenticated(
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    let (req, _user) = extract_and_insert_authuser(req).await?;

    Ok(next.run(req).await)
}

/// Admin-only guard.
pub async fn require_admin(
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    let (req, user) = extract_and_insert_authuser(req).await?;
    
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
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next,
    required_roles: &[&str],
    failure_msg: &str,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    let (req, user) = extract_and_insert_authuser(req).await?;
    
    let module_id = params.get("module_id")
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Missing or invalid module_id"))
        ))?;

    if user.0.admin {
        return Ok(next.run(req).await);
    }

    if user_has_any_role(user.0.sub, module_id, required_roles).await {
        Ok(next.run(req).await)
    } else {
        Err((StatusCode::FORBIDDEN, Json(ApiResponse::error(failure_msg))))
    }
}

/// Guard for requiring lecturer access.
pub async fn require_lecturer(
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    require_role_base(
        Path(params),
        req,
        next,
        &["Lecturer"],
        "Lecturer access required for this module"
    ).await
}

/// Guard for requiring assistant lecturer access.
pub async fn require_assistant_lecturer(
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    require_role_base(
        Path(params),
        req,
        next,
        &["AssistantLecturer"],
        "Assistant lecturer access required for this module"
    ).await
}

/// Guard for requiring tutor access.
pub async fn require_tutor(
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    require_role_base(
        Path(params),
        req,
        next,
        &["Tutor"],
        "Tutor access required for this module"
    ).await
}

/// Guard for requiring student access.
pub async fn require_student(
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    require_role_base(
        Path(params),
        req,
        next,
        &["Student"],
        "Student access required for this module"
    ).await
}

/// Guard for requiring lecturer or assistant lecturer access.
pub async fn require_lecturer_or_assistant_lecturer(
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    require_role_base(
        Path(params),
        req,
        next,
        &["Lecturer", "AssistantLecturer"],
        "Lecturer or assistant lecturer access required for this module"
    ).await
}

/// Guard for requiring lecturer or tutor access.
/// TODO: Add ALs to this?
pub async fn require_lecturer_or_tutor(
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    require_role_base(
        Path(params),
        req,
        next,
        &["Lecturer", "Tutor"],
        "Lecturer or tutor access required for this module"
    ).await
}

/// Guard for requiring any assigned role (lecturer, tutor, student).
pub async fn require_assigned_to_module(
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    require_role_base(
        Path(params),
        req,
        next,
        &["Lecturer", "AssistantLecturer", "Tutor", "Student"],
        "User not assigned to this module"
    ).await
}

pub async fn require_ready_assignment(
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    let module_id = params.get("module_id")
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Missing or invalid module_id"))
        ))?;

    let assignment_id = params.get("assignment_id")
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Missing or invalid assignment_id"))
        ))?;

    if let Err(e) = AssignmentService::try_transition_to_ready(module_id, assignment_id).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(format!("Failed to transition assignment to ready: {}", e)))
        ));
    }

    let assignment = match AssignmentService::find_by_id(assignment_id).await {
        Ok(Some(a)) => a,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(format!(
                    "Assignment {} in Module {} not found.",
                    assignment_id, module_id
                ))),
            ));
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Database error while checking assignment")),
            ));
        }
    };

    if assignment.status.to_string() == "setup".to_string() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Assignment is still in Setup stage"))
        ));
    }

    Ok(next.run(req).await)
}

// --- Path ID Guards ---

async fn check_module_exists(
    module_id: i64,
) -> Result<(), (StatusCode, Json<ApiResponse<Empty>>)> {
    let found = ModuleService::find_by_id(module_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("Database error while checking module"))))?;

    if found.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(format!("Module {} not found.", module_id))),
        ));
    }
    Ok(())
}

async fn check_user_exists(
    user_id: i64,
) -> Result<(), (StatusCode, Json<ApiResponse<Empty>>)> {
    let found = UserService::find_by_id(user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("Database error while checking user"))))?;

    if found.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(format!("User {} not found.", user_id))),
        ));
    }
    Ok(())
}

async fn check_assignment_hierarchy(
    module_id: i64,
    assignment_id: i64,
) -> Result<(), (StatusCode, Json<ApiResponse<Empty>>)> {
    check_module_exists(module_id).await?;

    let found = AssignmentService::find_one(
        &vec![
            FilterParam::eq("id", assignment_id),
            FilterParam::eq("module_id", module_id),
        ],
        None,
    ).await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("Database error while checking assignment"))))?;

    if found.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(format!("Assignment {} in Module {} not found.", assignment_id, module_id))),
        ));
    }
    Ok(())
}

async fn check_task_hierarchy(
    module_id: i64,
    assignment_id: i64,
    task_id: i64,
) -> Result<(), (StatusCode, Json<ApiResponse<Empty>>)> {
    check_assignment_hierarchy(module_id, assignment_id).await?;

    let found = AssignmentTaskService::find_one(
        &vec![
            FilterParam::eq("id", task_id),
            FilterParam::eq("assignment_id", assignment_id),
        ],
        None,
    ).await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("Database error while checking task"))))?;     

    if found.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(format!("Task {} in Assignment {} not found.", task_id, assignment_id))),
        ));
    }
    Ok(())
}

async fn check_submission_hierarchy(
    module_id: i64,
    assignment_id: i64,
    submission_id: i64,
) -> Result<(), (StatusCode, Json<ApiResponse<Empty>>)> {
    check_assignment_hierarchy(module_id, assignment_id).await?;

    let found = AssignmentSubmissionService::find_one(
        &vec![
            FilterParam::eq("id", submission_id),
            FilterParam::eq("assignment_id", assignment_id),
        ],
        None,
    ).await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("Database error while checking submission"))))?;

    if found.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(format!("Submission {} in Assignment {} not found.", submission_id, assignment_id))),
        ));
    }
    Ok(())
}

async fn check_file_hierarchy(
    module_id: i64,
    assignment_id: i64,
    file_id: i64,
) -> Result<(), (StatusCode, Json<ApiResponse<Empty>>)> {
    check_assignment_hierarchy(module_id, assignment_id).await?;

    let found = AssignmentFileService::find_one(
        &vec![
            FilterParam::eq("id", file_id),
            FilterParam::eq("assignment_id", assignment_id),
        ],
        None,
    ).await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("Database error while checking file"))))?;

    if found.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(format!("File {} in Assignment {} not found.", file_id, assignment_id))),
        ));
    }
    Ok(())
}

pub async fn check_ticket_hierarchy(
    module_id: i64,
    assignment_id: i64,
    ticket_id: i64,
) -> Result<(), (StatusCode, Json<ApiResponse<Empty>>)> {
    check_assignment_hierarchy(module_id, assignment_id).await?;

    let found = TicketService::find_one(
        &vec![
            FilterParam::eq("id", ticket_id),
            FilterParam::eq("assignment_id", assignment_id),
        ],
        None,
    ).await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("Database error while checking ticket"))))?;

    if found.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(format!("Ticket {} in Assignment {} not found.", ticket_id, assignment_id))),
        ));
    }
    Ok(())
}

pub async fn check_message_hierarchy(
    module_id: i64,
    assignment_id: i64,
    ticket_id: i64,
    message_id: i64,
) -> Result<(), (StatusCode, Json<ApiResponse<Empty>>)> {
    check_ticket_hierarchy(module_id, assignment_id, ticket_id).await?;

    let found = TicketMessageService::find_one(
        &vec![
            FilterParam::eq("id", message_id),
            FilterParam::eq("ticket_id", ticket_id),
        ],
        None,
    ).await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("Database error while checking message"))))?;

    if found.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(format!("Message {} in Ticket {} not found.", message_id, ticket_id))),
        ));
    }
    Ok(())
}

pub async fn check_plagiarism_hierarchy(
    module_id: i64,
    assignment_id: i64,
    case_id: i64,
) -> Result<(), (StatusCode, Json<ApiResponse<Empty>>)> {
    check_assignment_hierarchy(module_id, assignment_id).await?;

    let found = PlagiarismCaseService::find_one(
        &vec![
            FilterParam::eq("id", case_id),
            FilterParam::eq("assignment_id", assignment_id),
        ],
        None,
    ).await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("Database error while checking plagiarism case"))))?;

    if found.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(format!("Plagiarism case {} in Assignment {} not found.", case_id, assignment_id))),
        ));
    }
    Ok(())
}

pub async fn check_announcement_hierarchy(
    module_id: i64,
    announcement_id: i64,
) -> Result<(), (StatusCode, Json<ApiResponse<Empty>>)> {
    check_module_exists(module_id).await?;

    let found = AnnouncementService::find_one(
        &vec![
            FilterParam::eq("id", announcement_id),
            FilterParam::eq("module_id", module_id),
        ],
        None,
    ).await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("Database error while checking announcement"))))?;

    if found.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(format!("Announcement {} in Module {} not found.", announcement_id, module_id))),
        ));
    }
    Ok(())
}

pub async fn validate_known_ids(
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let mut module_id: Option<i64>     = None;
    let mut assignment_id: Option<i64> = None;
    let mut task_id: Option<i64>       = None;
    let mut submission_id: Option<i64> = None;
    let mut file_id: Option<i64>       = None;
    let mut user_id: Option<i64>       = None;
    let mut ticket_id: Option<i64>     = None;
    let mut message_id: Option<i64>    = None;
    let mut case_id: Option<i64>       = None;
    let mut announcement_id: Option<i64> = None;

    for (key, raw) in &params {
        let id = raw.parse::<i64>().map_err(|_| {
            (StatusCode::BAD_REQUEST, Json(ApiResponse::<Empty>::error(format!("Invalid {}: '{}'. Must be an integer.", key, raw)))).into_response()
        })?;
        match key.as_str() {
            "module_id"     => module_id = Some(id),
            "assignment_id" => assignment_id = Some(id),
            "task_id"       => task_id = Some(id),
            "submission_id" => submission_id = Some(id),
            "file_id"       => file_id = Some(id),
            "user_id"       => user_id = Some(id),
            "ticket_id"     => ticket_id = Some(id),
            "case_id" => case_id = Some(id),
            "announcement_id" => announcement_id = Some(id),
            "message_id" => message_id = Some(id),
            _ => return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::<Empty>::error(format!("Unexpected parameter: '{}'.", key)))).into_response()),
        }
    }
    
    if let Some(uid) = user_id {
        check_user_exists(uid).await.map_err(|e| e.into_response())?;
    }
    if let Some(mid) = module_id {
        check_module_exists(mid).await.map_err(|e| e.into_response())?;
    }
    if let (Some(mid), Some(aid)) = (module_id, assignment_id) {
        check_assignment_hierarchy(mid, aid).await.map_err(|e| e.into_response())?;
    }
    if let (Some(mid), Some(aid), Some(tid)) = (module_id, assignment_id, task_id) {
        check_task_hierarchy(mid, aid, tid).await.map_err(|e| e.into_response())?;
    }
    if let (Some(mid), Some(aid), Some(sid)) = (module_id, assignment_id, submission_id) {
        check_submission_hierarchy(mid, aid, sid).await.map_err(|e| e.into_response())?;
    }
    if let (Some(mid), Some(aid), Some(fid)) = (module_id, assignment_id, file_id) {
        check_file_hierarchy(mid, aid, fid).await.map_err(|e| e.into_response())?;
    }
    if let (Some(mid), Some(aid), Some(tid)) = (module_id, assignment_id, ticket_id) {
        check_ticket_hierarchy(mid, aid, tid).await.map_err(|e| e.into_response())?;
    }
    if let (Some(mid), Some(aid), Some(sid)) = (module_id, assignment_id, case_id) {
        check_plagiarism_hierarchy(mid, aid, sid).await.map_err(|e| e.into_response())?;
    }
    if let (Some(mid), Some(ann_id)) = (module_id, announcement_id) {
        check_announcement_hierarchy(mid, ann_id).await.map_err(|e| e.into_response())?;
    }
    if let (Some(mid), Some(aid), Some(tid), Some(meid)) = (module_id, assignment_id, ticket_id, message_id) {
        check_message_hierarchy(mid, aid, tid, meid).await.map_err(|e| e.into_response())?;
    }

    Ok(next.run(req).await)
}

// TODO Write tests for this gaurd
pub async fn require_ticket_ws_access(
    Path(params): Path<HashMap<String, String>>,
    req: axum::http::Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    // Must be logged in (also inserts AuthUser into extensions)
    let (req, user) = extract_and_insert_authuser(req).await?;

    // ticket_id from path
    let ticket_id = params.get("ticket_id")
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Missing or invalid ticket_id")),
        ))?;

    // Load ticket -> get assignment_id and author
    let ticket = TicketService::find_by_id(ticket_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("Database error while checking ticket"))))?
        .ok_or((StatusCode::NOT_FOUND, Json(ApiResponse::error("Ticket not found"))))?;

    // Author can access
    if ticket.user_id == user.0.sub {
        return Ok(next.run(req).await);
    }

    // Admin can access
    if user.0.admin {
        return Ok(next.run(req).await);
    }

    // Resolve module via assignment -> module_id
    let assignment = AssignmentService::find_by_id(ticket.assignment_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("Database error while checking assignment"))))?
        .ok_or((StatusCode::NOT_FOUND, Json(ApiResponse::error("Assignment not found for ticket"))))?;

    let module_id = assignment.module_id;

    // Allow module staff (Lecturer, AssistantLecturer, Tutor)
    if user_has_any_role(user.0.sub, module_id, &["Lecturer", "AssistantLecturer", "Tutor"]).await {
        return Ok(next.run(req).await);
    }

    // Otherwise, deny
    Err((StatusCode::FORBIDDEN, Json(ApiResponse::error("Not allowed to access this ticket websocket"))))
}