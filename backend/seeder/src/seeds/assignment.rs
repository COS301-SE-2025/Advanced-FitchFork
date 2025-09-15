use crate::seed::Seeder;
use services::service::{Service, AppError};
use services::module::ModuleService;
use services::assignment::{AssignmentService, AssignmentType, CreateAssignment};
use chrono::Utc;
use std::pin::Pin;

pub struct AssignmentSeeder;

impl Seeder for AssignmentSeeder {
    fn seed<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            let modules = ModuleService::find_all(
                &vec![],
                &vec![],
                None,
            ).await?;

            for m in &modules {
                if m.id == 9999 || m.id == 9998 || m.id == 10003 {
                    continue;
                }
                for i in 0..2 {
                    let now = Utc::now();
                    AssignmentService::create(
                        CreateAssignment{
                            id: None,
                            module_id: m.id,
                            name: format!("Assignment {i}"),
                            description: Some("Auto seeded".to_string()),
                            assignment_type: AssignmentType::Practical,
                            available_from: now,
                            due_date: now,
                        }
                    ).await?;
                }
            }
            let now = Utc::now();
            AssignmentService::create(
                CreateAssignment{
                    id: Some(9999),
                    module_id: 9999,
                    name: format!("Special Assignment"),
                    description: Some("Used for test zip execution".to_string()),
                    assignment_type: AssignmentType::Practical,
                    available_from: now,
                    due_date: now + chrono::Duration::days(7),
                }
            ).await?;

            AssignmentService::create(
                CreateAssignment{
                    id: Some(9998),
                    module_id: 9998,
                    name: format!("Special Assignment"),
                    description: Some("Used for test zip execution".to_string()),
                    assignment_type: AssignmentType::Practical,
                    available_from: now,
                    due_date: now + chrono::Duration::days(7),
                }
            ).await?;

            AssignmentService::create(
                CreateAssignment{
                    id: Some(10003),
                    module_id: 10003,
                    name: format!("Plagiarism Assignment"),
                    description: Some("Assignment used to show plagiarism cases".to_string()),
                    assignment_type: AssignmentType::Practical,
                    available_from: now,
                    due_date: now + chrono::Duration::days(7),
                }
            ).await?;

            AssignmentService::create(
                CreateAssignment{
                    id: Some(10004),
                    module_id: 10004,
                    name: format!("GATLAM Assignment"),
                    description: Some("Assignment used to show GATLAM".to_string()),
                    assignment_type: AssignmentType::Practical,
                    available_from: now,
                    due_date: now + chrono::Duration::days(7),
                }
            ).await?;

            Ok(())
        })
    }
}
