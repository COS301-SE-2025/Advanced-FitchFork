use crate::seed::run_seeder;
use crate::seed::Seeder;
use crate::seeds::{
    assignment::AssignmentSeeder, assignment_file::AssignmentFileSeeder,
    assignment_submission::AssignmentSubmissionSeeder, module::ModuleSeeder,
    submission_file::SubmissionFileSeeder, user::UserSeeder, user_role::UserRoleSeeder,
};

mod seed;
mod seeds;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let db = db::connect().await;

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
        (Box::new(SubmissionFileSeeder), "SubmissionFile"),
    ] {
        run_seeder(&*seeder, name, &db).await;
    }
}
