use crate::seed::Seeder;
use db::models::{assignment, assignment_task::Model as AssignmentTaskModel};
use rand::seq::SliceRandom;
use sea_orm::{DatabaseConnection, EntityTrait};

pub struct AssignmentTaskSeeder;

#[async_trait::async_trait]
impl Seeder for AssignmentTaskSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        // Fetch all assignments
        let assignments = assignment::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch assignments");

        if assignments.is_empty() {
            panic!("No assignments found — at least one assignment must exist to seed assignment_tasks");
        }

        let dummy_commands = vec![
            "fake command 1",
            "fake command 2",
            "fake command 3",
            "fake command 4",
            "fake command 5",
        ];

        for assignment in &assignments {
            let task_count = 2; // Number of tasks per assignment

            for i in 0..task_count {
                let task_number = i + 1;
                let command = dummy_commands
                    .choose(&mut rand::thread_rng())
                    .unwrap_or(&"echo 'Hello World'")
                    .to_string();

                match AssignmentTaskModel::create(db, assignment.id, task_number, &command).await {
                    Ok(_task) => {
                        // Optionally log or handle success
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to create assignment_task for assignment {} task {}: {}",
                            assignment.id, task_number, e
                        );
                    }
                }
            }
        }
    }
}
