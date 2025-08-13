use crate::seed::Seeder;
use db::models::{assignment, assignment_task::Model as AssignmentTaskModel};
use rand::seq::SliceRandom;
use sea_orm::{DatabaseConnection, EntityTrait};
use std::pin::Pin;

pub struct AssignmentTaskSeeder;

impl Seeder for AssignmentTaskSeeder {
    fn seed<'a>(&'a self, db: &'a DatabaseConnection) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
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
                if assignment.id == 9999 || assignment.id == 9998 {
                    continue;
                }
                let task_count = 2; // Number of tasks per assignment

                for i in 0..task_count {
                    let task_number = i + 1;
                    let command = dummy_commands
                        .choose(&mut rand::thread_rng())
                        .unwrap_or(&"echo 'Hello World'")
                        .to_string();

                    match AssignmentTaskModel::create(db, assignment.id, task_number, "Untitled Task", &command).await {
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
            let special_assignment_id: i64 = 9999;

            let special_tasks = vec![(1, "make task1"), (2, "make task2"), (3, "make task3")];

            for (task_number, command) in special_tasks {
                match db::models::assignment_task::Model::create(
                    db,
                    special_assignment_id,
                    task_number,
                    "Untitled Task",
                    command,
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => eprintln!(
                        "Failed to create special assignment task {}: {}",
                        task_number, e
                    ),
                }
            }

            let special_assignment_id2: i64 = 9998;

            let special_tasks2 = vec![(1, "make task1"), (2, "make task2"), (3, "make task3")];

            for (task_number, command) in special_tasks2 {
                match db::models::assignment_task::Model::create(
                    db,
                    special_assignment_id2,
                    task_number,
                    "Untitled Task",
                    command,
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => eprintln!(
                        "Failed to create special assignment task {}: {}",
                        task_number, e
                    ),
                }
            }
        })
    }
}
