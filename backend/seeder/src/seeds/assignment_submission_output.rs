use crate::seed::Seeder;
use db::models::{
    assignment_submission::Entity as AssignmentSubmission,
    assignment_submission_output::Model as AssignmentSubmissionOutputModel, assignment_task,
};

use sea_orm::{DatabaseConnection, EntityTrait};

pub struct AssignmentSubmissionOutputSeeder;

#[async_trait::async_trait]
impl Seeder for AssignmentSubmissionOutputSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        let tasks = assignment_task::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch assignment tasks");

        let submissions = AssignmentSubmission::find()
            .all(db)
            .await
            .expect("Failed to fetch assignment submissions");

        if tasks.is_empty() || submissions.is_empty() {
            panic!("Tasks or submissions are missing — cannot seed submission outputs");
        }

        for submission in &submissions {
            if submission.assignment_id == 9999
                || submission.assignment_id == 9998
                || submission.assignment_id == 10003
            {
                continue;
            }
            let relevant_tasks: Vec<_> = tasks
                .iter()
                .filter(|t| t.assignment_id == submission.assignment_id)
                .collect();

            for task in relevant_tasks {
                let dummy_filename = "submission_output.txt";
                let dummy_content = format!(
                    "Generated submission output for submission {} assignment {} task {}",
                    submission.id, submission.assignment_id, task.task_number
                );

                match AssignmentSubmissionOutputModel::save_file(
                    db,
                    task.id,
                    submission.id,
                    dummy_filename,
                    dummy_content.as_bytes(),
                )
                .await
                {
                    Ok(_) => {
                        // Optionally log success
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to save submission output for submission {} assignment {} task {}: {}",
                            submission.id, submission.assignment_id, task.task_number, e
                        );
                    }
                }
            }
        }
    }
}
