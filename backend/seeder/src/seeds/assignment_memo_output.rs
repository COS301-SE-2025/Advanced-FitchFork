use crate::seed::Seeder;
use services::assignment::AssignmentService;
use services::assignment_memo_output::{AssignmentMemoOutputService, CreateAssignmentMemoOutput};
use services::assignment_task::AssignmentTaskService;
use services::service::{AppError, Service};
use std::pin::Pin;

pub struct AssignmentMemoOutputSeeder;

impl Seeder for AssignmentMemoOutputSeeder {
    fn seed<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            // Fetch all assignments
            let assignments = AssignmentService::find_all(&vec![], &vec![], None).await?;
            let tasks = AssignmentTaskService::find_all(&vec![], &vec![], None).await?;

            if assignments.is_empty() || tasks.is_empty() {
                panic!("Assignments or tasks are missing — cannot seed memo outputs");
            }

            for assignment in &assignments {
                if assignment.id == 9999 || assignment.id == 9998 || assignment.id == 10003 {
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

                    AssignmentMemoOutputService::create(CreateAssignmentMemoOutput {
                        assignment_id: assignment.id,
                        task_id: task.id,
                        filename: dummy_filename.to_string(),
                        bytes: dummy_content.as_bytes().to_vec(),
                    })
                    .await?;
                }
            }

            Ok(())
        })
    }
}
