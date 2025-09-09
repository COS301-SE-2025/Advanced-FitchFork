use crate::seed::Seeder;
use crate::seed::run_seeder;
use crate::seeds::announcement::AnnouncementSeeder;
use crate::seeds::{
    assignment::AssignmentSeeder, assignment_file::AssignmentFileSeeder,
    assignment_interpreter::AssignmentInterpreterSeeder,
    assignment_memo_output::AssignmentMemoOutputSeeder,
    assignment_overwrite_file::AssignmentOverwriteFileSeeder,
    assignment_submission::AssignmentSubmissionSeeder,
    assignment_submission_output::AssignmentSubmissionOutputSeeder,
    assignment_task::AssignmentTaskSeeder, module::ModuleSeeder,
    plagiarism_case::PlagiarismCaseSeeder, tickets::TicketSeeder, user::UserSeeder,
    user_role::UserRoleSeeder,
};

mod seed;
mod seeds;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    for (seeder, name) in [
        (
            Box::new(UserSeeder) as Box<dyn Seeder + Send + Sync>,
            "User",
        ),
        (Box::new(ModuleSeeder), "Module"),
        (Box::new(AssignmentSeeder), "Assignment"),
        (Box::new(UserRoleSeeder), "UserRole"),
        (Box::new(AnnouncementSeeder), "Announcement"), 
        (Box::new(AssignmentFileSeeder), "AssignmentFile"),
        (Box::new(AssignmentSubmissionSeeder), "AssignmentSubmission"),
        (Box::new(AssignmentTaskSeeder), "AssignmentTask"),
        (Box::new(AssignmentMemoOutputSeeder), "AssignmentMemoOutput"),
        (Box::new(PlagiarismCaseSeeder), "Plagiarism"),
        (Box::new(AssignmentSubmissionOutputSeeder),"AssignmentSubmissionOutput"),
        (Box::new(AssignmentOverwriteFileSeeder),"AssignmentOverwriteFile"),
        (Box::new(AssignmentInterpreterSeeder), "AssignmentInterpreter"),
        (Box::new(TicketSeeder), "Ticket"),
    ] {
        run_seeder(&*seeder, name).await;
    }
}