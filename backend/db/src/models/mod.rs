pub mod user;
pub mod module;
pub mod assignment;
pub mod assignment_file;
pub mod user_module_role;

pub use user::Entity as User;
pub use module::Entity as Module;
pub use assignment::Entity as Assignment;
pub use assignment_file::Entity as AssignmentFile;
pub use user_module_role::Entity as UserModuleRole;
