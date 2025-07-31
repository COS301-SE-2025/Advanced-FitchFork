use crate::seed::run_seeder;
use crate::seed::Seeder;
use crate::seeds::{
    assignment::AssignmentSeeder, assignment_file::AssignmentFileSeeder,
    assignment_memo_output::AssignmentMemoOutputSeeder,
    assignment_overwrite_file::AssignmentOverwriteFileSeeder,
    assignment_submission::AssignmentSubmissionSeeder,
    assignment_submission_output::AssignmentSubmissionOutputSeeder,
    assignment_task::AssignmentTaskSeeder, module::ModuleSeeder, user::UserSeeder,
    user_role::UserRoleSeeder,
};
use util::app_state::app_state::AppState;

mod seed;
mod seeds;

#[tokio::main]
async fn main() {
    let _ = AppState::init(false);

    for (seeder, name) in [
        (
            Box::new(UserSeeder) as Box<dyn Seeder + Send + Sync>,
            "User",
        ),
        (Box::new(ModuleSeeder), "Module"),
        (Box::new(AssignmentSeeder), "Assignment"),
        (Box::new(UserRoleSeeder), "UserRole"),
        (Box::new(AssignmentFileSeeder), "AssignmentFile"),
        (Box::new(AssignmentSubmissionSeeder), "AssignmentSubmission"),
        (Box::new(AssignmentTaskSeeder), "AssignmentTask"),
        (Box::new(AssignmentMemoOutputSeeder), "AssignmentMemoOutput"),
        (
            Box::new(AssignmentSubmissionOutputSeeder),
            "AssignmentSubmissionOutput",
        ),
        (
            Box::new(AssignmentOverwriteFileSeeder),
            "AssignmentOverwriteFile",
        ),
    ] {
        run_seeder(&*seeder, name).await;
    }
}
