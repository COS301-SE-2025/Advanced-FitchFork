use chrono::{DateTime, Utc};
use crate::models::assignment::{AssignmentType, Status};

#[derive(Debug, Clone, Default)]
pub struct UserFilter {
    pub id: Option<i64>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub admin: Option<bool>,
    pub query: Option<String>,
}

impl UserFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    pub fn with_admin(mut self, admin: bool) -> Self {
        self.admin = Some(admin);
        self
    }

    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct ModuleFilter {
    pub id: Option<i64>,
    pub code: Option<String>,
    pub year: Option<i32>,
    pub description: Option<String>,
    pub credits: Option<i32>,
    pub query: Option<String>,
}

impl ModuleFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }

    pub fn with_year(mut self, year: i32) -> Self {
        self.year = Some(year);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_credits(mut self, credits: i32) -> Self {
        self.credits = Some(credits);
        self
    }

    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct AssignmentFilter {
    pub id: Option<i64>,
    pub module_id: Option<i64>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub assignment_type: Option<AssignmentType>,
    pub status: Option<Status>,
    pub available_before: Option<DateTime<Utc>>,
    pub available_after: Option<DateTime<Utc>>,
    pub due_before: Option<DateTime<Utc>>,
    pub due_after: Option<DateTime<Utc>>,
    pub query: Option<String>,
}

impl AssignmentFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_module_id(mut self, module_id: i64) -> Self {
        self.module_id = Some(module_id);
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_assignment_type(mut self, assignment_type: AssignmentType) -> Self {
        self.assignment_type = Some(assignment_type);
        self
    }

    pub fn with_status(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_available_before(mut self, date: DateTime<Utc>) -> Self {
        self.available_before = Some(date);
        self
    }

    pub fn with_available_after(mut self, date: DateTime<Utc>) -> Self {
        self.available_after = Some(date);
        self
    }

    pub fn with_due_before(mut self, date: DateTime<Utc>) -> Self {
        self.due_before = Some(date);
        self
    }

    pub fn with_due_after(mut self, date: DateTime<Utc>) -> Self {
        self.due_after = Some(date);
        self
    }

    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }
}

use crate::models::user_module_role::Role;

#[derive(Debug, Clone, Default)]
pub struct UserModuleRoleFilter {
    pub user_id: Option<i64>,
    pub module_id: Option<i64>,
    pub role: Option<Role>,
}

impl UserModuleRoleFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_user_id(mut self, user_id: i64) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_module_id(mut self, module_id: i64) -> Self {
        self.module_id = Some(module_id);
        self
    }

    pub fn with_role(mut self, role: Role) -> Self {
        self.role = Some(role);
        self
    }
}

use crate::models::assignment_file::FileType;

#[derive(Debug, Clone, Default)]
pub struct AssignmentFileFilter {
    pub id: Option<i64>,
    pub assignment_id: Option<i64>,
    pub filename: Option<String>,
    pub file_type: Option<FileType>,
    pub query: Option<String>,
}

impl AssignmentFileFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_assignment_id(mut self, assignment_id: i64) -> Self {
        self.assignment_id = Some(assignment_id);
        self
    }

    pub fn with_filename(mut self, filename: String) -> Self {
        self.filename = Some(filename);
        self
    }

    pub fn with_file_type(mut self, file_type: FileType) -> Self {
        self.file_type = Some(file_type);
        self
    }

    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }
}
#[derive(Debug, Clone, Default)]
pub struct AssignmentSubmissionFilter {
    pub id: Option<i64>,
    pub assignment_id: Option<i64>,
    pub user_id: Option<i64>,
    pub attempt: Option<i64>,
    pub filename: Option<String>,
    pub file_hash: Option<String>,
    pub is_practice: Option<bool>,
    pub query: Option<String>,
}

impl AssignmentSubmissionFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_assignment_id(mut self, assignment_id: i64) -> Self {
        self.assignment_id = Some(assignment_id);
        self
    }

    pub fn with_user_id(mut self, user_id: i64) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_attempt(mut self, attempt: i64) -> Self {
        self.attempt = Some(attempt);
        self
    }

    pub fn with_filename(mut self, filename: String) -> Self {
        self.filename = Some(filename);
        self
    }

    pub fn with_file_hash(mut self, file_hash: String) -> Self {
        self.file_hash = Some(file_hash);
        self
    }

    pub fn with_is_practice(mut self, is_practice: bool) -> Self {
        self.is_practice = Some(is_practice);
        self
    }

    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }
}
#[derive(Debug, Clone, Default)]
pub struct AssignmentTaskFilter {
    pub id: Option<i64>,
    pub assignment_id: Option<i64>,
    pub task_number: Option<i64>,
    pub name: Option<String>,
    pub command: Option<String>,
    pub query: Option<String>,
}

impl AssignmentTaskFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_assignment_id(mut self, assignment_id: i64) -> Self {
        self.assignment_id = Some(assignment_id);
        self
    }

    pub fn with_task_number(mut self, task_number: i64) -> Self {
        self.task_number = Some(task_number);
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_command(mut self, command: String) -> Self {
        self.command = Some(command);
        self
    }

    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }
}
#[derive(Debug, Clone, Default)]
pub struct AssignmentMemoOutputFilter {
    pub id: Option<i64>,
    pub assignment_id: Option<i64>,
    pub task_id: Option<i64>,
    pub query: Option<String>,
}

impl AssignmentMemoOutputFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_assignment_id(mut self, assignment_id: i64) -> Self {
        self.assignment_id = Some(assignment_id);
        self
    }

    pub fn with_task_id(mut self, task_id: i64) -> Self {
        self.task_id = Some(task_id);
        self
    }

    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct AssignmentSubmissionOutputFilter {
    pub id: Option<i64>,
    pub task_id: Option<i64>,
    pub submission_id: Option<i64>,
    pub query: Option<String>,
}

impl AssignmentSubmissionOutputFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_task_id(mut self, task_id: i64) -> Self {
        self.task_id = Some(task_id);
        self
    }

    pub fn with_submission_id(mut self, submission_id: i64) -> Self {
        self.submission_id = Some(submission_id);
        self
    }

    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct AssignmentOverwriteFileFilter {
    pub id: Option<i64>,
    pub assignment_id: Option<i64>,
    pub task_id: Option<i64>,
    pub filename: Option<String>,
    pub query: Option<String>,
}

impl AssignmentOverwriteFileFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_assignment_id(mut self, assignment_id: i64) -> Self {
        self.assignment_id = Some(assignment_id);
        self
    }

    pub fn with_task_id(mut self, task_id: i64) -> Self {
        self.task_id = Some(task_id);
        self
    }

    pub fn with_filename(mut self, filename: String) -> Self {
        self.filename = Some(filename);
        self
    }

    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct PasswordResetTokenFilter {
    pub id: Option<i64>,
    pub user_id: Option<i64>,
    pub token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub used: Option<bool>,
    pub query: Option<String>,
}

impl PasswordResetTokenFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_user_id(mut self, user_id: i64) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    pub fn with_expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn with_used(mut self, used: bool) -> Self {
        self.used = Some(used);
        self
    }

    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }
}
use crate::models::plagiarism_case::Status as PlagiarismStatus;

#[derive(Debug, Clone, Default)]
pub struct PlagiarismCaseFilter {
    pub id: Option<i64>,
    pub assignment_id: Option<i64>,
    pub submission_id_1: Option<i64>,
    pub submission_id_2: Option<i64>,
    pub description: Option<String>,
    pub status: Option<PlagiarismStatus>,
    pub query: Option<String>,
}

impl PlagiarismCaseFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_assignment_id(mut self, assignment_id: i64) -> Self {
        self.assignment_id = Some(assignment_id);
        self
    }

    pub fn with_submission_id_1(mut self, submission_id_1: i64) -> Self {
        self.submission_id_1 = Some(submission_id_1);
        self
    }

    pub fn with_submission_id_2(mut self, submission_id_2: i64) -> Self {
        self.submission_id_2 = Some(submission_id_2);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_status(mut self, status: PlagiarismStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }
}
use crate::models::tickets::TicketStatus;

#[derive(Debug, Clone, Default)]
pub struct TicketFilter {
    pub id: Option<i64>,
    pub assignment_id: Option<i64>,
    pub user_id: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TicketStatus>,
    pub query: Option<String>,
}

impl TicketFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_assignment_id(mut self, assignment_id: i64) -> Self {
        self.assignment_id = Some(assignment_id);
        self
    }

    pub fn with_user_id(mut self, user_id: i64) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_status(mut self, status: TicketStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct TicketMessageFilter {
    pub id: Option<i64>,
    pub ticket_id: Option<i64>,
    pub user_id: Option<i64>,
    pub content: Option<String>,
    pub query: Option<String>,
}

impl TicketMessageFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_ticket_id(mut self, ticket_id: i64) -> Self {
        self.ticket_id = Some(ticket_id);
        self
    }

    pub fn with_user_id(mut self, user_id: i64) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }

    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }
}