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
use sea_orm::DatabaseConnection;
use std::{collections::HashMap, net::{IpAddr, Ipv4Addr}};
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use db::models::{
    module::Entity as ModuleEntity,
    plagiarism_case::{Entity as PlagiarismEntity, Column as PlagiarismColumn},
    assignment::{Entity as AssignmentEntity, Column as AssignmentColumn},
    assignment_task::{Entity as TaskEntity, Column as TaskColumn},
    assignment_submission::{Entity as SubmissionEntity, Column as SubmissionColumn},
    assignment_file::{Entity as FileEntity, Column as FileColumn},
    user::Entity as UserEntity,
    attendance_session::{Entity as AttendanceSessionEntity, Column as AttendanceSessionColumn},
    moss_report::{Entity as MossReportEntity, Column as MossReportColumn},
    user
};

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
        &vec![],
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
        &vec![],
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
        &vec![],
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
        &vec![],
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
        &vec![],
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
        &vec![],
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
        &vec![],
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
        &vec![],
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

async fn check_attendance_session_hierarchy(
    module_id: i32,
    session_id: i32,
    db: &DatabaseConnection,
) -> Result<(), (StatusCode, Json<ApiResponse<Empty>>)> {
    // Ensure module exists
    check_module_exists(module_id, db).await?;

    let found = AttendanceSessionEntity::find()
        .filter(AttendanceSessionColumn::Id.eq(session_id))
        .filter(AttendanceSessionColumn::ModuleId.eq(module_id))
        .one(db)
        .await
        .map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR,
             Json(ApiResponse::error("Database error while checking attendance session")))
        })?;

    if found.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(format!(
                "Attendance session {} in Module {} not found.", session_id, module_id
            ))),
        ));
    }

    Ok(())
}

async fn check_moss_report_hierarchy(
    module_id: i32,
    assignment_id: i32,
    report_id: i32,
    db: &DatabaseConnection,
) -> Result<(), (StatusCode, Json<ApiResponse<Empty>>)> {
    // Ensure module & assignment exist / relate
    check_assignment_hierarchy(module_id, assignment_id, db).await?;

    // Ensure the report belongs to the assignment
    let found = MossReportEntity::find()
        .filter(MossReportColumn::Id.eq(report_id))
        .filter(MossReportColumn::AssignmentId.eq(assignment_id))
        .one(db)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Database error while checking MOSS report")),
            )
        })?;

    if found.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(format!(
                "MOSS report {} in Assignment {} not found.",
                report_id, assignment_id
            ))),
        ));
    }
    Ok(())
}

pub async fn validate_known_ids(
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let db = app_state.db();

    let mut module_id: Option<i32>       = None;
    let mut assignment_id: Option<i32>   = None;
    let mut task_id: Option<i32>         = None;
    let mut submission_id: Option<i32>   = None;
    let mut file_id: Option<i32>         = None;
    let mut user_id: Option<i32>         = None;
    let mut ticket_id: Option<i32>       = None;
    let mut message_id: Option<i32>      = None;
    let mut case_id: Option<i32>         = None;
    let mut announcement_id: Option<i32> = None;
    let mut session_id: Option<i32>      = None;
    let mut report_id: Option<i32>     = None;

    for (key, raw) in &params {
        match key.as_str() {
            // numeric ids → parse i32 (existing behavior)
            "module_id"     |
            "assignment_id" |
            "task_id"       |
            "submission_id" |
            "file_id"       |
            "user_id"       |
            "ticket_id"     |
            "case_id"       |
            "announcement_id" |
            "message_id"    |
            "session_id" |
            "report_id"  => {
                let id = raw.parse::<i32>().map_err(|_| {
                    (StatusCode::BAD_REQUEST, Json(ApiResponse::<Empty>::error(
                        format!("Invalid {}: '{}'. Must be an integer.", key, raw)
                    ))).into_response()
                })?;
                match key.as_str() {
                    "module_id"       => module_id = Some(id),
                    "assignment_id"   => assignment_id = Some(id),
                    "task_id"         => task_id = Some(id),
                    "submission_id"   => submission_id = Some(id),
                    "file_id"         => file_id = Some(id),
                    "user_id"         => user_id = Some(id),
                    "ticket_id"       => ticket_id = Some(id),
                    "case_id"         => case_id = Some(id),
                    "announcement_id" => announcement_id = Some(id),
                    "message_id"      => message_id = Some(id),
                    "session_id"      => session_id = Some(id),
                    "report_id"       => report_id = Some(id), 
                    _ => {}
                }
            }

            // anything else → still reject
            _ => {
                return Err((StatusCode::BAD_REQUEST, Json(ApiResponse::<Empty>::error(
                    format!("Unexpected parameter: '{}'.", key)
                ))).into_response());
            }
        }
    }

    // existing checks (unchanged)
    if let Some(uid) = user_id { check_user_exists(uid, db).await.map_err(|e| e.into_response())?; }
    if let Some(mid) = module_id { check_module_exists(mid, db).await.map_err(|e| e.into_response())?; }
    if let (Some(mid), Some(aid)) = (module_id, assignment_id) { check_assignment_hierarchy(mid, aid, db).await.map_err(|e| e.into_response())?; }
    if let (Some(mid), Some(aid), Some(tid)) = (module_id, assignment_id, task_id) { check_task_hierarchy(mid, aid, tid, db).await.map_err(|e| e.into_response())?; }
    if let (Some(mid), Some(aid), Some(sid)) = (module_id, assignment_id, submission_id) { check_submission_hierarchy(mid, aid, sid, db).await.map_err(|e| e.into_response())?; }
    if let (Some(mid), Some(aid), Some(fid)) = (module_id, assignment_id, file_id) { check_file_hierarchy(mid, aid, fid, db).await.map_err(|e| e.into_response())?; }
    if let (Some(mid), Some(aid), Some(tid)) = (module_id, assignment_id, ticket_id) { check_ticket_hierarchy(mid, aid, tid, db).await.map_err(|e| e.into_response())?; }
    if let (Some(mid), Some(aid), Some(cid)) = (module_id, assignment_id, case_id) { check_plagiarism_hierarchy(mid, aid, cid, db).await.map_err(|e| e.into_response())?; }
    if let (Some(mid), Some(ann_id)) = (module_id, announcement_id) { check_announcement_hierarchy(mid, ann_id, db).await.map_err(|e| e.into_response())?; }
    if let (Some(mid), Some(aid), Some(tid), Some(meid)) = (module_id, assignment_id, ticket_id, message_id) { check_message_hierarchy(mid, aid, tid, meid, db).await.map_err(|e| e.into_response())?; }
    if let (Some(mid), Some(sid)) = (module_id, session_id) { check_attendance_session_hierarchy(mid, sid, db).await.map_err(|e| e.into_response())?; }
    if let (Some(mid), Some(aid), Some(rid)) = (module_id, assignment_id, report_id) {
        check_moss_report_hierarchy(mid, aid, rid, db).await.map_err(|e| e.into_response())?;
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

pub async fn require_attendance_ws_access(
    State(app_state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<Empty>>)> {
    let db = app_state.db();

    // Must be logged in (also inserts AuthUser into extensions)
    let (req, user) = extract_and_insert_authuser(req).await?;

    // Parse session_id from path
    let session_id = params.get("session_id")
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("Missing or invalid session_id")),
        ))?;

    // Load the attendance session to resolve module_id
    use db::models::attendance_session::{Entity as SessionEntity, Column as SessionCol};
    use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};

    let sess = SessionEntity::find()
        .filter(SessionCol::Id.eq(session_id))
        .one(db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("Database error while checking session"))))?
        .ok_or((StatusCode::NOT_FOUND, Json(ApiResponse::error("Attendance session not found"))))?;

    let module_id = sess.module_id;

    // Admin always allowed
    if user.0.admin {
        return Ok(next.run(req).await);
    }

    // Allow Lecturer or AssistantLecturer of this module
    if user_has_any_role(db, user.0.sub, module_id, &["Lecturer", "AssistantLecturer"]).await {
        return Ok(next.run(req).await);
    }

    Err((StatusCode::FORBIDDEN, Json(ApiResponse::error("Not allowed to access this attendance session websocket"))))
}

/// Guard that enforces assignment security **for students only**:
/// 1) Checks client IP against allowlist (if configured)
/// 2) Then verifies PIN, if required
///
/// Admin + staff (Lecturer, AssistantLecturer, Tutor) are bypassed.
/// Path must include `{module_id}` and `{assignment_id}`.
/// When required, PIN is read from `x-assignment-pin` header.
pub async fn require_assignment_access(
    State(app_state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
    req: axum::http::Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<()>>)> {
    let db = app_state.db();

    // Auth user must be inserted by an upstream guard
    let user = req
        .extensions()
        .get::<AuthUser>()
        .cloned()
        .ok_or((StatusCode::UNAUTHORIZED, Json(ApiResponse::error("Authentication required"))))?;

    let module_id = params.get("module_id")
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or((StatusCode::BAD_REQUEST, Json(ApiResponse::error("Missing or invalid module_id"))))?;

    let assignment_id = params.get("assignment_id")
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or((StatusCode::BAD_REQUEST, Json(ApiResponse::error("Missing or invalid assignment_id"))))?;

    // --- Bypass for admin/staff ---
    if user.0.admin
        || user_has_any_role(db, user.0.sub, module_id, &["Lecturer", "AssistantLecturer", "Tutor"]).await
    {
        return Ok(next.run(req).await);
    }

    // Only enforce for students; unknown roles fall through to next
    let is_student = user_has_any_role(db, user.0.sub, module_id, &["Student"]).await;
    if !is_student {
        return Ok(next.run(req).await);
    }

    // Load assignment for security config
    let assignment = AssignmentEntity::find()
        .filter(AssignmentColumn::Id.eq(assignment_id))
        .filter(AssignmentColumn::ModuleId.eq(module_id))
        .one(db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("Database error while loading assignment"))))?
        .ok_or((StatusCode::NOT_FOUND, Json(ApiResponse::error("Assignment not found"))))?;

    // ---- 1) IP allowlist check (students only) ----
    let mut client_ip = req
        .extensions()
        .get::<IpAddr>()
        .cloned()
        .unwrap_or_else(|| "127.0.0.1".parse().unwrap());

    // Normalize IPv6 loopback (::1) to IPv4 loopback (127.0.0.1) for deterministic tests & configs
    if let IpAddr::V6(v6) = client_ip {
        if v6.is_loopback() {
            client_ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
        }
    }

    if !assignment.ip_allowed(client_ip) {
        return Err((StatusCode::FORBIDDEN, Json(ApiResponse::error("IP not allowed"))));
    }

    // ---- 2) PIN check (students only, if enabled) ----
    if assignment.password_required_for_students() {
        let maybe_pin = req.headers().get("x-assignment-pin").and_then(|h| h.to_str().ok());
        match maybe_pin {
            Some(pin) if assignment.verify_password_from_config(pin) => { /* ok */ }
            _ => {
                // Signal to the client that verification is needed
                return Err((StatusCode::FORBIDDEN, Json(ApiResponse::error("PIN required"))));
            }
        }
    }

    Ok(next.run(req).await)
}