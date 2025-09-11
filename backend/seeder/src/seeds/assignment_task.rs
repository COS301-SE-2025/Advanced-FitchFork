use crate::seed::Seeder;
use services::service::{Service, AppError};
use services::assignment::AssignmentService;
use services::assignment_task::{AssignmentTaskService, CreateAssignmentTask};
use rand::seq::SliceRandom;
use std::pin::Pin;

pub struct AssignmentTaskSeeder;

impl Seeder for AssignmentTaskSeeder {
    fn seed<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            // Fetch all assignments
            let assignments = AssignmentService::find_all(&[], None).await?;
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

                    AssignmentTaskService::create(
                        CreateAssignmentTask{
                            assignment_id: assignment.id,
                            task_number: task_number,
                            name: "Untitled Task".to_string(),
                            command: command,
                            code_coverage: false,
                        }
                    ).await?;
                }
            }

            let special_tasks = vec![(1, "make task1"), (2, "make task2"), (3, "make task3")];
            for (task_number, command) in special_tasks {
                AssignmentTaskService::create(
                    CreateAssignmentTask{
                        assignment_id: 9999,
                        task_number: task_number,
                        name: "Untitled Task".to_string(),
                        command: command.to_string(),
                        code_coverage: false,
                    }
                ).await?;
            }

            let special_tasks2 = vec![
                (1, "make task1", false),
                (2, "make task2", false),
                (3, "make task3", false),
                (4, "make task4", true),
            ];
            for (task_number, command, code_coverage) in special_tasks2 {
                AssignmentTaskService::create(
                    CreateAssignmentTask{
                        assignment_id: 9998,
                        task_number: task_number,
                        name: "Untitled Task".to_string(),
                        command: command.to_string(),
                        code_coverage: code_coverage,
                    }
                ).await?;
            }

            let special_tasks2 = vec![(1, "make task1")];
            for (task_number, command) in special_tasks2 {
                AssignmentTaskService::create(
                    CreateAssignmentTask{
                        assignment_id: 10003,
                        task_number: task_number,
                        name: "Untitled Task".to_string(),
                        command: command.to_string(),
                        code_coverage: false,
                    }
                ).await?;

                let special_tasks3 = vec![(1, "make task1")];
                for (task_number, command) in special_tasks3 {
                    AssignmentTaskService::create(
                        CreateAssignmentTask{
                            assignment_id: 10004,
                            task_number: task_number,
                            name: "Task to run code".to_string(),
                            command: command.to_string(),
                            code_coverage: false,
                        }
                    ).await?;
                }
            }

            Ok(())
        })
    }
}
