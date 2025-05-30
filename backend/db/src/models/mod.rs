pub mod assignment;
pub mod assignment_file;
pub mod assignment_submission;
pub mod module;
pub mod submission_file;
pub mod user;
pub mod user_module_role;

pub use assignment::Entity as Assignment;
pub use assignment_file::Entity as AssignmentFile;
pub use assignment_submission::Entity as AssignmentSubmission;
pub use module::Entity as Module;
pub use submission_file::Entity as SubmissionFile;
pub use user::Entity as User;
pub use user_module_role::Entity as UserModuleRole;
