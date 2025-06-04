use crate::seed::Seeder;
use db::models::{
    assignment, assignment_submission, assignment_submission::Entity as AssignmentSubmission,
    assignment_submission_output::Model as AssignmentSubmissionOutputModel, assignment_task, user,
};

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub struct AssignmentSubmissionOutputSeeder;

#[async_trait::async_trait]
impl Seeder for AssignmentSubmissionOutputSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        let assignments = assignment::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch assignments");

        let tasks = assignment_task::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch assignment tasks");

        let users = user::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch users");

        if assignments.is_empty() || tasks.is_empty() || users.is_empty() {
            panic!("Assignments, tasks, or users are missing — cannot seed submission outputs");
        }

        for assignment in &assignments {
            let relevant_tasks: Vec<_> = tasks
                .iter()
                .filter(|t| t.assignment_id == assignment.id)
                .collect();

            for task in &relevant_tasks {
                for user in &users {
                    // Check if this user has a submission for the assignment
                    let submission = AssignmentSubmission::find()
                        .filter(
                            assignment_submission::Column::AssignmentId
                                .eq(assignment.id)
                                .and(assignment_submission::Column::UserId.eq(user.id)),
                        )
                        .one(db)
                        .await
                        .expect("DB query failed");

                    if let Some(_submission) = submission {
                        let dummy_filename = "submission_output.txt";
                        let dummy_content = format!(
                            "Generated submission output for user {} assignment {} task {}",
                            user.id, assignment.id, task.task_number
                        );

                        match AssignmentSubmissionOutputModel::save_file(
                            db,
                            assignment.id,
                            task.task_number,
                            user.id,
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
                                    "Failed to save submission output for user {} assignment {} task {}: {}",
                                    user.id, assignment.id, task.task_number, e
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
