use crate::seed::Seeder;
use db::{
    get_connection,
    models::{assignment, assignment_overwrite_file::Model as AssignmentOverwriteFileModel, assignment_task}
};
use sea_orm::EntityTrait;

pub struct AssignmentOverwriteFileSeeder;

#[async_trait::async_trait]
impl Seeder for AssignmentOverwriteFileSeeder {
    async fn seed(&self) {
        let db = get_connection().await;

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
            panic!("Assignments or tasks are missing — cannot seed overwrite files");
        }

        for assignment in &assignments {
            if assignment.id == 9999 || assignment.id == 9998 {
                continue;
            }
            let relevant_tasks: Vec<_> = tasks
                .iter()
                .filter(|t| t.assignment_id == assignment.id)
                .collect();

            for task in relevant_tasks {
                let dummy_filename = "overwrite_file.txt";
                let dummy_content = format!(
                    "Generated overwrite file for assignment {} task {}",
                    assignment.id, task.id
                );

                match AssignmentOverwriteFileModel::save_file(
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
                            "Failed to save overwrite file for assignment {} task {}: {}",
                            assignment.id, task.id, e
                        );
                    }
                }
            }
        }
    }
}