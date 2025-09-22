use crate::seed::Seeder;
use services::assignment_submission::AssignmentSubmissionService;
use services::assignment_submission_output::{
    AssignmentSubmissionOutputService, CreateAssignmentSubmissionOutput,
};
use services::assignment_task::AssignmentTaskService;
use services::service::{AppError, Service};
use std::pin::Pin;

pub struct AssignmentSubmissionOutputSeeder;

impl Seeder for AssignmentSubmissionOutputSeeder {
    fn seed<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            let tasks = AssignmentTaskService::find_all(&vec![], &vec![], None).await?;
            let submissions = AssignmentSubmissionService::find_all(&vec![], &vec![], None).await?;

            if tasks.is_empty() || submissions.is_empty() {
                panic!("Tasks or submissions are missing — cannot seed submission outputs");
            }

            for submission in &submissions {
                if submission.assignment_id == 9999
                    || submission.assignment_id == 9998
                    || submission.assignment_id == 10003
                    || submission.assignment_id == 10004
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

                    AssignmentSubmissionOutputService::create(CreateAssignmentSubmissionOutput {
                        task_id: task.id,
                        submission_id: submission.id,
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
