use crate::seed::Seeder;
use services::service::{Service, AppError};
use services::assignment::AssignmentService;
use services::assignment_task::AssignmentTaskService;
use services::assignment_overwrite_file::{AssignmentOverwriteFileService, CreateAssignmentOverwriteFile};
use std::pin::Pin;

pub struct AssignmentOverwriteFileSeeder;

impl Seeder for AssignmentOverwriteFileSeeder {
    fn seed<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            // Fetch all assignments
            let assignments = AssignmentService::find_all(&[], None).await?;
            let tasks = AssignmentTaskService::find_all(&[], None).await?;

            if assignments.is_empty() || tasks.is_empty() {
                panic!("Assignments or tasks are missing — cannot seed overwrite files");
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
                    let dummy_filename = "overwrite_file.txt";
                    let dummy_content = format!(
                        "Generated overwrite file for assignment {} task {}",
                        assignment.id, task.id
                    );

                    AssignmentOverwriteFileService::create(
                        CreateAssignmentOverwriteFile{
                            assignment_id: assignment.id,
                            task_id: task.id,
                            filename: dummy_filename.to_string(),
                            bytes: dummy_content.as_bytes().to_vec(),
                        }
                    ).await?;
                }
            }

            Ok(())
        })
    }
}
