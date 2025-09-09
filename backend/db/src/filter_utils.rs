// use chrono::{DateTime, Utc};
// use crate::models::assignment::{AssignmentType, Status};
// use crate::comparisons::Comparison;
// use crate::models::user_module_role::Role;
// use crate::models::plagiarism_case::Status as PlagiarismStatus;
// use crate::models::tickets::TicketStatus;
// use crate::models::assignment_file::FileType;

// #[derive(Debug, Clone, Default)]
// pub struct UserFilter {
//     pub id: Option<Comparison<i64>>,
//     pub username: Option<Comparison<String>>,
//     pub email: Option<Comparison<String>>,
//     pub admin: Option<Comparison<bool>>,
// }

// // impl UserFilter {
// //     pub fn new() -> Self {
// //         Self::default()
// //     }

// //     pub fn with_id(mut self, id: Comparison<i64>) -> Self {
// //         self.id = Some(id);
// //         self
// //     }

// //     pub fn with_username(mut self, username: Comparison<String>) -> Self {
// //         self.username = Some(username);
// //         self
// //     }

// //     pub fn with_email(mut self, email: Comparison<String>) -> Self {
// //         self.email = Some(email);
// //         self
// //     }

// //     pub fn with_admin(mut self, admin: Comparison<bool>) -> Self {
// //         self.admin = Some(admin);
// //         self
// //     }
// // }

// #[derive(Debug, Clone, Default)]
// pub struct ModuleFilter {
//     pub id: Option<Comparison<i64>>,
//     pub code: Option<Comparison<String>>,
//     pub year: Option<Comparison<i64>>,
//     pub description: Option<Comparison<String>>,
//     pub credits: Option<Comparison<i64>>,
// }

// // impl ModuleFilter {
// //     pub fn new() -> Self {
// //         Self::default()
// //     }

// //     pub fn with_id(mut self, id: Comparison<i64>) -> Self {
// //         self.id = Some(id);
// //         self
// //     }

// //     pub fn with_code(mut self, code: Comparison<String>) -> Self {
// //         self.code = Some(code);
// //         self
// //     }

// //     pub fn with_year(mut self, year: Comparison<i64>) -> Self {
// //         self.year = Some(year);
// //         self
// //     }

// //     pub fn with_description(mut self, description: Comparison<String>) -> Self {
// //         self.description = Some(description);
// //         self
// //     }

// //     pub fn with_credits(mut self, credits: Comparison<i64>) -> Self {
// //         self.credits = Some(credits);
// //         self
// //     }
// // }

// #[derive(Debug, Clone, Default)]
// pub struct AssignmentFilter {
//     pub id: Option<Comparison<i64>>,
//     pub module_id: Option<Comparison<i64>>,
//     pub name: Option<Comparison<String>>,
//     pub description: Option<Comparison<String>>,
//     pub assignment_type: Option<Comparison<AssignmentType>>,
//     pub status: Option<Comparison<Status>>,
//     pub available_before: Option<Comparison<DateTime<Utc>>>,
//     pub available_after: Option<Comparison<DateTime<Utc>>>,
//     pub due_before: Option<Comparison<DateTime<Utc>>>,
//     pub due_after: Option<Comparison<DateTime<Utc>>>,
// }

// // impl AssignmentFilter {
// //     pub fn new() -> Self {
// //         Self::default()
// //     }

// //     pub fn with_id(mut self, id: Comparison<i64>) -> Self {
// //         self.id = Some(id);
// //         self
// //     }

// //     pub fn with_module_id(mut self, module_id: Comparison<i64>) -> Self {
// //         self.module_id = Some(module_id);
// //         self
// //     }

// //     pub fn with_name(mut self, name: Comparison<String>) -> Self {
// //         self.name = Some(name);
// //         self
// //     }

// //     pub fn with_description(mut self, description: Comparison<String>) -> Self {
// //         self.description = Some(description);
// //         self
// //     }

// //     pub fn with_assignment_type(mut self, assignment_type: Comparison<AssignmentType>) -> Self {
// //         self.assignment_type = Some(assignment_type);
// //         self
// //     }

// //     pub fn with_status(mut self, status: Comparison<Status>) -> Self {
// //         self.status = Some(status);
// //         self
// //     }

// //     pub fn with_available_before(mut self, date: Comparison<DateTime<Utc>>) -> Self {
// //         self.available_before = Some(date);
// //         self
// //     }

// //     pub fn with_available_after(mut self, date: Comparison<DateTime<Utc>>) -> Self {
// //         self.available_after = Some(date);
// //         self
// //     }

// //     pub fn with_due_before(mut self, date: Comparison<DateTime<Utc>>) -> Self {
// //         self.due_before = Some(date);
// //         self
// //     }

// //     pub fn with_due_after(mut self, date: Comparison<DateTime<Utc>>) -> Self {
// //         self.due_after = Some(date);
// //         self
// //     }
// // }

// #[derive(Debug, Clone, Default)]
// pub struct UserModuleRoleFilter {
//     pub user_id: Option<Comparison<i64>>,
//     pub module_id: Option<Comparison<i64>>,
//     pub role: Option<Comparison<Role>>,
// }

// // impl UserModuleRoleFilter {
// //     pub fn new() -> Self {
// //         Self::default()
// //     }

// //     pub fn with_user_id(mut self, user_id: Comparison<i64>) -> Self {
// //         self.user_id = Some(user_id);
// //         self
// //     }

// //     pub fn with_module_id(mut self, module_id: Comparison<i64>) -> Self {
// //         self.module_id = Some(module_id);
// //         self
// //     }

// //     pub fn with_role(mut self, role: Comparison<Role>) -> Self {
// //         self.role = Some(role);
// //         self
// //     }
// // }

// #[derive(Debug, Clone, Default)]
// pub struct AssignmentFileFilter {
//     pub id: Option<Comparison<i64>>,
//     pub assignment_id: Option<Comparison<i64>>,
//     pub filename: Option<Comparison<String>>,
//     pub file_type: Option<Comparison<FileType>>,
// }

// // impl AssignmentFileFilter {
// //     pub fn new() -> Self {
// //         Self::default()
// //     }

// //     pub fn with_id(mut self, id: Comparison<i64>) -> Self {
// //         self.id = Some(id);
// //         self
// //     }

// //     pub fn with_assignment_id(mut self, assignment_id: Comparison<i64>) -> Self {
// //         self.assignment_id = Some(assignment_id);
// //         self
// //     }

// //     pub fn with_filename(mut self, filename: Comparison<String>) -> Self {
// //         self.filename = Some(filename);
// //         self
// //     }

// //     pub fn with_file_type(mut self, file_type: Comparison<FileType>) -> Self {
// //         self.file_type = Some(file_type);
// //         self
// //     }
// // }
// #[derive(Debug, Clone, Default)]
// pub struct AssignmentSubmissionFilter {
//     pub id: Option<Comparison<i64>>,
//     pub assignment_id: Option<Comparison<i64>>,
//     pub user_id: Option<Comparison<i64>>,
//     pub attempt: Option<Comparison<i64>>,
//     pub is_practice: Option<Comparison<bool>>,
//     pub ignored: Option<Comparison<bool>>,
//     pub filename: Option<Comparison<String>>,
//     pub file_hash: Option<Comparison<String>>,
// }

// // impl AssignmentSubmissionFilter {
// //     pub fn new() -> Self {
// //         Self::default()
// //     }

// //     pub fn with_id(mut self, id: Comparison<i64>) -> Self {
// //         self.id = Some(id);
// //         self
// //     }

// //     pub fn with_assignment_id(mut self, assignment_id: Comparison<i64>) -> Self {
// //         self.assignment_id = Some(assignment_id);
// //         self
// //     }

// //     pub fn with_user_id(mut self, user_id: Comparison<i64>) -> Self {
// //         self.user_id = Some(user_id);
// //         self
// //     }

// //     pub fn with_attempt(mut self, attempt: Comparison<i64>) -> Self {
// //         self.attempt = Some(attempt);
// //         self
// //     }

// //     pub fn with_is_practice(mut self, is_practice: Comparison<bool>) -> Self {
// //         self.is_practice = Some(is_practice);
// //         self
// //     }

// //     pub fn with_ignored(mut self, ignored: Comparison<bool>) -> Self {
// //         self.ignored = Some(ignored);
// //         self
// //     }

// //     pub fn with_filename(mut self, filename: Comparison<String>) -> Self {
// //         self.filename = Some(filename);
// //         self
// //     }

// //     pub fn with_file_hash(mut self, file_hash: Comparison<String>) -> Self {
// //         self.file_hash = Some(file_hash);
// //         self
// //     }
// // }

// #[derive(Debug, Clone, Default)]
// pub struct AssignmentTaskFilter {
//     pub id: Option<Comparison<i64>>,
//     pub assignment_id: Option<Comparison<i64>>,
//     pub task_number: Option<Comparison<i64>>,
//     pub name: Option<Comparison<String>>,
//     pub command: Option<Comparison<String>>,
// }

// // impl AssignmentTaskFilter {
// //     pub fn new() -> Self {
// //         Self::default()
// //     }

// //     pub fn with_id(mut self, id: Comparison<i64>) -> Self {
// //         self.id = Some(id);
// //         self
// //     }

// //     pub fn with_assignment_id(mut self, assignment_id: Comparison<i64>) -> Self {
// //         self.assignment_id = Some(assignment_id);
// //         self
// //     }

// //     pub fn with_task_number(mut self, task_number: Comparison<i64>) -> Self {
// //         self.task_number = Some(task_number);
// //         self
// //     }

// //     pub fn with_name(mut self, name: Comparison<String>) -> Self {
// //         self.name = Some(name);
// //         self
// //     }

// //     pub fn with_command(mut self, command: Comparison<String>) -> Self {
// //         self.command = Some(command);
// //         self
// //     }
// // }

// #[derive(Debug, Clone, Default)]
// pub struct AssignmentMemoOutputFilter {
//     pub id: Option<Comparison<i64>>,
//     pub assignment_id: Option<Comparison<i64>>,
//     pub task_id: Option<Comparison<i64>>,
// }

// // impl AssignmentMemoOutputFilter {
// //     pub fn new() -> Self {
// //         Self::default()
// //     }

// //     pub fn with_id(mut self, id: Comparison<i64>) -> Self {
// //         self.id = Some(id);
// //         self
// //     }

// //     pub fn with_assignment_id(mut self, assignment_id: Comparison<i64>) -> Self {
// //         self.assignment_id = Some(assignment_id);
// //         self
// //     }

// //     pub fn with_task_id(mut self, task_id: Comparison<i64>) -> Self {
// //         self.task_id = Some(task_id);
// //         self
// //     }
// // }

// #[derive(Debug, Clone, Default)]
// pub struct AssignmentSubmissionOutputFilter {
//     pub id: Option<Comparison<i64>>,
//     pub task_id: Option<Comparison<i64>>,
//     pub submission_id: Option<Comparison<i64>>,
// }

// // impl AssignmentSubmissionOutputFilter {
// //     pub fn new() -> Self {
// //         Self::default()
// //     }

// //     pub fn with_id(mut self, id: Comparison<i64>) -> Self {
// //         self.id = Some(id);
// //         self
// //     }

// //     pub fn with_task_id(mut self, task_id: Comparison<i64>) -> Self {
// //         self.task_id = Some(task_id);
// //         self
// //     }

// //     pub fn with_submission_id(mut self, submission_id: Comparison<i64>) -> Self {
// //         self.submission_id = Some(submission_id);
// //         self
// //     }
// // }

// #[derive(Debug, Clone, Default)]
// pub struct AssignmentOverwriteFileFilter {
//     pub id: Option<Comparison<i64>>,
//     pub assignment_id: Option<Comparison<i64>>,
//     pub task_id: Option<Comparison<i64>>,
//     pub filename: Option<Comparison<String>>,
// }

// // impl AssignmentOverwriteFileFilter {
// //     pub fn new() -> Self {
// //         Self::default()
// //     }

// //     pub fn with_id(mut self, id: Comparison<i64>) -> Self {
// //         self.id = Some(id);
// //         self
// //     }

// //     pub fn with_assignment_id(mut self, assignment_id: Comparison<i64>) -> Self {
// //         self.assignment_id = Some(assignment_id);
// //         self
// //     }

// //     pub fn with_task_id(mut self, task_id: Comparison<i64>) -> Self {
// //         self.task_id = Some(task_id);
// //         self
// //     }

// //     pub fn with_filename(mut self, filename: Comparison<String>) -> Self {
// //         self.filename = Some(filename);
// //         self
// //     }
// // }

// #[derive(Debug, Clone, Default)]
// pub struct PasswordResetTokenFilter {
//     pub id: Option<Comparison<i64>>,
//     pub user_id: Option<Comparison<i64>>,
//     pub token: Option<Comparison<String>>,
//     pub expires_at: Option<Comparison<DateTime<Utc>>>,
//     pub used: Option<Comparison<bool>>,
// }

// // impl PasswordResetTokenFilter {
// //     pub fn new() -> Self {
// //         Self::default()
// //     }

// //     pub fn with_id(mut self, id: Comparison<i64>) -> Self {
// //         self.id = Some(id);
// //         self
// //     }

// //     pub fn with_user_id(mut self, user_id: Comparison<i64>) -> Self {
// //         self.user_id = Some(user_id);
// //         self
// //     }

// //     pub fn with_token(mut self, token: Comparison<String>) -> Self {
// //         self.token = Some(token);
// //         self
// //     }

// //     pub fn with_expires_at(mut self, expires_at: Comparison<DateTime<Utc>>) -> Self {
// //         self.expires_at = Some(expires_at);
// //         self
// //     }

// //     pub fn with_used(mut self, used: Comparison<bool>) -> Self {
// //         self.used = Some(used);
// //         self
// //     }
// // }

// #[derive(Debug, Clone, Default)]
// pub struct PlagiarismCaseFilter {
//     pub id: Option<Comparison<i64>>,
//     pub assignment_id: Option<Comparison<i64>>,
//     pub submission_id_1: Option<Comparison<i64>>,
//     pub submission_id_2: Option<Comparison<i64>>,
//     pub description: Option<Comparison<String>>,
//     pub status: Option<Comparison<PlagiarismStatus>>,
// }

// // impl PlagiarismCaseFilter {
// //     pub fn new() -> Self {
// //         Self::default()
// //     }

// //     pub fn with_id(mut self, id: Comparison<i64>) -> Self {
// //         self.id = Some(id);
// //         self
// //     }

// //     pub fn with_assignment_id(mut self, assignment_id: Comparison<i64>) -> Self {
// //         self.assignment_id = Some(assignment_id);
// //         self
// //     }

// //     pub fn with_submission_id_1(mut self, submission_id_1: Comparison<i64>) -> Self {
// //         self.submission_id_1 = Some(submission_id_1);
// //         self
// //     }

// //     pub fn with_submission_id_2(mut self, submission_id_2: Comparison<i64>) -> Self {
// //         self.submission_id_2 = Some(submission_id_2);
// //         self
// //     }

// //     pub fn with_description(mut self, description: Comparison<String>) -> Self {
// //         self.description = Some(description);
// //         self
// //     }

// //     pub fn with_status(mut self, status: Comparison<PlagiarismStatus>) -> Self {
// //         self.status = Some(status);
// //         self
// //     }
// // }

// #[derive(Debug, Clone, Default)]
// pub struct TicketFilter {
//     pub id: Option<Comparison<i64>>,
//     pub assignment_id: Option<Comparison<i64>>,
//     pub user_id: Option<Comparison<i64>>,
//     pub title: Option<Comparison<String>>,
//     pub description: Option<Comparison<String>>,
//     pub status: Option<Comparison<TicketStatus>>,
// }

// // impl TicketFilter {
// //     pub fn new() -> Self {
// //         Self::default()
// //     }

// //     pub fn with_id(mut self, id: Comparison<i64>) -> Self {
// //         self.id = Some(id);
// //         self
// //     }

// //     pub fn with_assignment_id(mut self, assignment_id: Comparison<i64>) -> Self {
// //         self.assignment_id = Some(assignment_id);
// //         self
// //     }

// //     pub fn with_user_id(mut self, user_id: Comparison<i64>) -> Self {
// //         self.user_id = Some(user_id);
// //         self
// //     }

// //     pub fn with_title(mut self, title: Comparison<String>) -> Self {
// //         self.title = Some(title);
// //         self
// //     }

// //     pub fn with_description(mut self, description: Comparison<String>) -> Self {
// //         self.description = Some(description);
// //         self
// //     }

// //     pub fn with_status(mut self, status: Comparison<TicketStatus>) -> Self {
// //         self.status = Some(status);
// //         self
// //     }
// // }

// #[derive(Debug, Clone, Default)]
// pub struct TicketMessageFilter {
//     pub id: Option<Comparison<i64>>,
//     pub ticket_id: Option<Comparison<i64>>,
//     pub user_id: Option<Comparison<i64>>,
//     pub content: Option<Comparison<String>>,
// }

// // impl TicketMessageFilter {
// //     pub fn new() -> Self {
// //         Self::default()
// //     }

// //     pub fn with_id(mut self, id: Comparison<i64>) -> Self {
// //         self.id = Some(id);
// //         self
// //     }

// //     pub fn with_ticket_id(mut self, ticket_id: Comparison<i64>) -> Self {
// //         self.ticket_id = Some(ticket_id);
// //         self
// //     }

// //     pub fn with_user_id(mut self, user_id: Comparison<i64>) -> Self {
// //         self.user_id = Some(user_id);
// //         self
// //     }

// //     pub fn with_content(mut self, content: Comparison<String>) -> Self {
// //         self.content = Some(content);
// //         self
// //     }
// // }

// #[derive(Debug, Clone, Default)]
// pub struct AnnouncementFilter {
//     pub id: Option<Comparison<i64>>,
//     pub module_id: Option<Comparison<i64>>,
//     pub user_id: Option<Comparison<i64>>,
//     pub title: Option<Comparison<String>>,
//     pub body: Option<Comparison<String>>,
//     pub pinned: Option<Comparison<bool>>,
// }

// // impl AnnouncementFilter {
// //     pub fn new() -> Self {
// //         Self::default()
// //     }

// //     pub fn with_id(mut self, id: Comparison<i64>) -> Self {
// //         self.id = Some(id);
// //         self
// //     }

// //     pub fn with_module_id(mut self, module_id: Comparison<i64>) -> Self {
// //         self.module_id = Some(module_id);
// //         self
// //     }

// //     pub fn with_user_id(mut self, user_id: Comparison<i64>) -> Self {
// //         self.user_id = Some(user_id);
// //         self
// //     }

// //     pub fn with_title(mut self, title: Comparison<String>) -> Self {
// //         self.title = Some(title);
// //         self
// //     }

// //     pub fn with_body(mut self, body: Comparison<String>) -> Self {
// //         self.body = Some(body);
// //         self
// //     }

// //     pub fn with_pinned(mut self, pinned: Comparison<bool>) -> Self {
// //         self.pinned = Some(pinned);
// //         self
// //     }
// // }

// #[derive(Debug, Clone, Default)]
// pub struct AssignmentInterpreterFilter {
//     pub id: Option<Comparison<i64>>,
//     pub assignment_id: Option<Comparison<i64>>,
//     pub filename: Option<Comparison<String>>,
//     pub path: Option<Comparison<String>>,
//     pub command: Option<Comparison<String>>,
// }

// // impl AssignmentInterpreterFilter {
// //     pub fn new() -> Self {
// //         Self::default()
// //     }

// //     pub fn with_id(mut self, id: Comparison<i64>) -> Self {
// //         self.id = Some(id);
// //         self
// //     }

// //     pub fn with_assignment_id(mut self, assignment_id: Comparison<i64>) -> Self {
// //         self.assignment_id = Some(assignment_id);
// //         self
// //     }

// //     pub fn with_filename(mut self, filename: Comparison<String>) -> Self {
// //         self.filename = Some(filename);
// //         self
// //     }

// //     pub fn with_path(mut self, path: Comparison<String>) -> Self {
// //         self.path = Some(path);
// //         self
// //     }

// //     pub fn with_command(mut self, command: Comparison<String>) -> Self {
// //         self.command = Some(command);
// //         self
// //     }
// // }

use sea_orm::{Condition, QueryOrder, ColumnTrait, DbErr, prelude::Expr};
use util::filters::{FilterParam, FilterValue, CompareOp};

pub struct FilterUtils;

impl FilterUtils {
    /// Apply a single filter parameter to a condition using any SeaORM column
    pub fn apply_filter<C>(
        condition: Condition, 
        column: C, 
        filter_param: &FilterParam
    ) -> Result<Condition, DbErr> 
    where 
        C: ColumnTrait
    {
        match (&filter_param.value, &filter_param.operator) {
            // String operations
            (FilterValue::String(value), CompareOp::Eq) => {
                Ok(condition.add(column.eq(value)))
            },
            (FilterValue::String(value), CompareOp::NotEq) => {
                Ok(condition.add(column.ne(value)))
            },
            (FilterValue::String(value), CompareOp::Like) => {
                let pattern = format!("%{}%", value.to_lowercase());
                Ok(condition.add(
                    Expr::cust(&format!("LOWER({})", column.as_str())).like(&pattern)
                ))
            },
            (FilterValue::String(value), CompareOp::Gt) => {
                Ok(condition.add(column.gt(value)))
            },
            (FilterValue::String(value), CompareOp::Gte) => {
                Ok(condition.add(column.gte(value)))
            },
            (FilterValue::String(value), CompareOp::Lt) => {
                Ok(condition.add(column.lt(value)))
            },
            (FilterValue::String(value), CompareOp::Lte) => {
                Ok(condition.add(column.lte(value)))
            },
            
            // Integer operations
            (FilterValue::Int(value), CompareOp::Eq) => {
                Ok(condition.add(column.eq(*value)))
            },
            (FilterValue::Int(value), CompareOp::NotEq) => {
                Ok(condition.add(column.ne(*value)))
            },
            (FilterValue::Int(value), CompareOp::Gt) => {
                Ok(condition.add(column.gt(*value)))
            },
            (FilterValue::Int(value), CompareOp::Gte) => {
                Ok(condition.add(column.gte(*value)))
            },
            (FilterValue::Int(value), CompareOp::Lt) => {
                Ok(condition.add(column.lt(*value)))
            },
            (FilterValue::Int(value), CompareOp::Lte) => {
                Ok(condition.add(column.lte(*value)))
            },
            (FilterValue::Int(value), CompareOp::Like) => {
                let pattern = format!("%{}%", value);
                Ok(condition.add(column.like(&pattern)))
            },
            
            // Boolean operations
            (FilterValue::Bool(value), CompareOp::Eq) => {
                Ok(condition.add(column.eq(*value)))
            },
            (FilterValue::Bool(value), CompareOp::NotEq) => {
                Ok(condition.add(column.ne(*value)))
            },
            
            // DateTime operations
            (FilterValue::DateTime(value), CompareOp::Eq) => {
                Ok(condition.add(column.eq(*value)))
            },
            (FilterValue::DateTime(value), CompareOp::NotEq) => {
                Ok(condition.add(column.ne(*value)))
            },
            (FilterValue::DateTime(value), CompareOp::Gt) => {
                Ok(condition.add(column.gt(*value)))
            },
            (FilterValue::DateTime(value), CompareOp::Gte) => {
                Ok(condition.add(column.gte(*value)))
            },
            (FilterValue::DateTime(value), CompareOp::Lt) => {
                Ok(condition.add(column.lt(*value)))
            },
            (FilterValue::DateTime(value), CompareOp::Lte) => {
                Ok(condition.add(column.lte(*value)))
            },
            
            // Invalid combinations
            (FilterValue::Bool(_), CompareOp::Gt | CompareOp::Gte | CompareOp::Lt | CompareOp::Lte | CompareOp::Like) => {
                Err(DbErr::Custom(format!("Invalid operator {:?} for boolean value", filter_param.operator)))
            },
            (FilterValue::DateTime(_), CompareOp::Like) => {
                Err(DbErr::Custom("LIKE operator not supported for DateTime values".to_string()))
            },
        }
    }

    /// Generic method to apply all filter parameters with proper column resolution
    pub fn apply_all_filters<C>(
        filter_params: &[FilterParam],
        column_resolver: impl Fn(&str) -> Result<C, DbErr>
    ) -> Result<Condition, DbErr>
    where
        C: ColumnTrait
    {
        let mut condition = Condition::all();
        
        for filter_param in filter_params {
            let column = column_resolver(&filter_param.column)?;
            condition = Self::apply_filter(condition, column, filter_param)?;
        }
        
        Ok(condition)
    }
}

// Generic sorting utility
pub struct SortUtils;

impl SortUtils {
    pub fn apply_sorting<E, C>(
        mut query: sea_orm::Select<E>, 
        sort_by: Option<String>,
        column_resolver: impl Fn(&str) -> Result<C, DbErr>
    ) -> Result<sea_orm::Select<E>, DbErr>
    where
        E: sea_orm::EntityTrait,
        C: ColumnTrait
    {
        if let Some(sort) = sort_by {
            let (column_name, asc) = if sort.starts_with('-') {
                (&sort[1..], false)
            } else {
                (sort.as_str(), true)
            };

            let column = column_resolver(column_name)?;
            query = if asc {
                query.order_by_asc(column)
            } else {
                query.order_by_desc(column)
            };
        }
        Ok(query)
    }
}