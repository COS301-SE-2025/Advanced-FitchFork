use crate::seed::Seeder;
use db::models::{assignment, assignment_submission::Model as AssignmentSubmissionModel, user};
use sea_orm::{DatabaseConnection, EntityTrait};

pub struct AssignmentSubmissionSeeder;

#[async_trait::async_trait]
impl Seeder for AssignmentSubmissionSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        // Fetch all assignments and users
        let assignments = assignment::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch assignments");

        let users = user::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch users");

        if users.is_empty() {
            panic!("No users found — at least one user must exist to seed assignment_submissions");
        }
        for assignment in &assignments {
            for user in &users {
                for counter in 1..=2 {
                    // Dummy original filename and content for seeding
                    let dummy_filename = "submission.txt";
                    let dummy_content = format!(
                        "Dummy submission content for assignment {} by user {}",
                        assignment.id, user.id
                    );

                    // Use the Model's save_file method to create DB entry and write file
                    match AssignmentSubmissionModel::save_file(
                        db,
                        assignment.id,
                        user.id,
                        counter,
                        dummy_filename,
                        dummy_content.as_bytes(),
                    )
                    .await
                    {
                        Ok(_file) => {
                            // Optionally log or handle success
                        }
                        Err(e) => {
                            eprintln!(
                            "Failed to save assignment_submission file for assignment {} user {}: {}",
                            assignment.id, user.id, e
                        );
                        }
                    }
                }
            }
        }
    }
}
