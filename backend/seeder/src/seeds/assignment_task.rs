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
            panic!(
                "No assignments found — at least one assignment must exist to seed assignment_tasks"
            );
        }

        let dummy_commands = vec![
            "fake command 1",
            "fake command 2",
            "fake command 3",
            "fake command 4",
            "fake command 5",
        ];

        for assignment in &assignments {
            if assignment.id == 9999
                || assignment.id == 9998
                || assignment.id == 10003
                || assignment.id == 10004
            {
                continue;
            }
            let task_count = 2; // Number of tasks per assignment

            for i in 0..task_count {
                let task_number = i + 1;
                let command = dummy_commands
                    .choose(&mut rand::thread_rng())
                    .unwrap_or(&"echo 'Hello World'")
                    .to_string();

                match AssignmentTaskModel::create(
                    db,
                    assignment.id,
                    task_number,
                    "Untitled Task",
                    &command,
                    false,
                    false,
                )
                .await
                {
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

        let special_tasks = vec![
            (1, "make task1", false),
            (2, "make task2", false),
            (3, "make task3", false),
            (4, "make task4", true),
        ];

        for (task_number, command, code_coverage) in special_tasks {
            match db::models::assignment_task::Model::create(
                db,
                special_assignment_id,
                task_number,
                "Untitled Task",
                command,
                code_coverage,
                false,
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

        let special_tasks2 = vec![
            (1, "make task1", false, false),
            (2, "make task2", false, true),
            (3, "make task3", false, true),
            (4, "make task4", true, false),
        ];

        for (task_number, command, code_coverage, valgrind) in special_tasks2 {
            match db::models::assignment_task::Model::create(
                db,
                special_assignment_id2,
                task_number,
                "Untitled Task",
                command,
                code_coverage,
                valgrind,
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

        let plag_assignment_id: i64 = 10003;

        let special_tasks2 = vec![(1, "make task1")];

        for (task_number, command) in special_tasks2 {
            match db::models::assignment_task::Model::create(
                db,
                plag_assignment_id,
                task_number,
                "Task to run code",
                command,
                false,
                false,
            )
            .await
            {
                Ok(_) => {}
                Err(e) => eprintln!(
                    "Failed to create special assignment task {}: {}",
                    task_number, e
                ),
            }

            let special_tasks3 = vec![(1, "make task1")];

            for (task_number, command) in special_tasks3 {
                match db::models::assignment_task::Model::create(
                    db,
                    10004,
                    task_number,
                    "Task to run code",
                    command,
                    false,
                    false,
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
        }
    }
}
