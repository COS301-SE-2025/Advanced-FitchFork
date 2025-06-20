use crate::seed::Seeder;
use db::models::{
    assignment, assignment_memo_output::Model as AssignmentMemoOutputModel, assignment_task,
};
use sea_orm::{DatabaseConnection, EntityTrait};

pub struct AssignmentMemoOutputSeeder;

#[async_trait::async_trait]
impl Seeder for AssignmentMemoOutputSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        // Fetch all assignments
        let assignments = assignment::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch assignments");

        // Fetch all assignment tasks
        let tasks = assignment_task::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch assignment tasks");

        if assignments.is_empty() || tasks.is_empty() {
            panic!("Assignments or tasks are missing — cannot seed memo outputs");
        }

        for assignment in &assignments {
            if assignment.id == 9999 {
                continue;
            }
            let relevant_tasks: Vec<_> = tasks
                .iter()
                .filter(|t| t.assignment_id == assignment.id)
                .collect();

            for task in relevant_tasks {
                let dummy_filename = "memo_output.txt";
                let dummy_content = format!(
                    "Generated memo output for assignment {} task {}",
                    assignment.id, task.id
                );

                match AssignmentMemoOutputModel::save_file(
                    db,
                    assignment.id,
                    task.id,
                    dummy_filename,
                    dummy_content.as_bytes(),
                )
                .await
                {
                    Ok(_output) => {
                        // Optionally log success
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to save memo output for assignment {} task {}: {}",
                            assignment.id, task.id, e
                        );
                    }
                }
            }
        }
    }
}
