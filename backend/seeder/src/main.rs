use crate::seed::Seeder;
use crate::seeds::{
    user::UserSeeder,
    module::ModuleSeeder,
    assignment::AssignmentSeeder,
    user_role::UserRoleSeeder,
    assignment_file::AssignmentFileSeeder,
};
use crate::seed::run_seeder;

mod seed;
mod seeds;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let db = db::connect().await;

    for (seeder, name) in [
        (Box::new(UserSeeder) as Box<dyn Seeder + Send + Sync>, "User"),
        (Box::new(ModuleSeeder), "Module"),
        (Box::new(AssignmentSeeder), "Assignment"),
        (Box::new(UserRoleSeeder), "UserRole"),
        (Box::new(AssignmentFileSeeder), "AssignmentFile"),
    ] {
        run_seeder(&*seeder, name, &db).await;
    }
}
