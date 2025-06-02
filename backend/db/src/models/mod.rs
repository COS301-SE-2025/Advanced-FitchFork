pub mod user_module_role;
pub mod assignment;
pub mod assignment_file;
pub mod assignment_submission;
pub mod module;
pub mod user;
pub mod password_reset_token;

pub use user_module_role::Entity as UserModuleRole;
pub use assignment::Entity as Assignment;
pub use assignment_file::Entity as AssignmentFile;
pub use assignment_submission::Entity as AssignmentSubmission;
pub use module::Entity as Module;
pub use user::Entity as User;
pub use password_reset_token::Entity as PasswordResetToken;
