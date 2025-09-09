use crate::seed::Seeder;
use chrono::Utc;
use db::models::{
    assignment,
    assignment::{AssignmentType, Status},
    module,
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::pin::Pin;

pub struct AssignmentSeeder;

impl Seeder for AssignmentSeeder {
    fn seed<'a>(&'a self, db: &'a DatabaseConnection) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let modules = module::Entity::find()
                .all(db)
                .await
                .expect("Failed to fetch modules");

            for m in &modules {
                if m.id == 9999 || m.id == 9998 || m.id == 10003 {
                    continue;
                }
                for i in 0..2 {
                    let a = assignment::ActiveModel {
                        module_id: Set(m.id),
                        name: Set(format!("Assignment {i}")),
                        description: Set(Some("Auto seeded".to_string())),
                        assignment_type: Set(AssignmentType::Practical),
                        status: Set(Status::Setup),
                        available_from: Set(Utc::now()),
                        due_date: Set(Utc::now()),
                        created_at: Set(Utc::now()),
                        updated_at: Set(Utc::now()),
                        ..Default::default()
                    };
                    let _ = a.insert(db).await;
                }
            }

                let special_assignment = assignment::ActiveModel {
                    id: Set(9999),
                    module_id: Set(9999),
                    name: Set("Special Assignment".to_string()),
                    description: Set(Some("Used for test zip execution".to_string())),
                    assignment_type: Set(AssignmentType::Practical),
                    status: Set(Status::Setup),
                    available_from: Set(Utc::now()),
                    due_date: Set(Utc::now() + chrono::Duration::days(7)),
                    created_at: Set(Utc::now()),
                    updated_at: Set(Utc::now()),
                    ..Default::default()
                };

                let _ = special_assignment.insert(db).await;

                let special_assignment2 = assignment::ActiveModel {
                    id: Set(9998),
                    module_id: Set(9998),
                    name: Set("Special Assignment".to_string()),
                    description: Set(Some("Used for test zip execution".to_string())),
                    assignment_type: Set(AssignmentType::Practical),
                    status: Set(Status::Setup),
                    available_from: Set(Utc::now()),
                    due_date: Set(Utc::now() + chrono::Duration::days(7)),
                    created_at: Set(Utc::now()),
                    updated_at: Set(Utc::now()),
                    ..Default::default()
                };

            let _ = special_assignment2.insert(db).await;

            let plagiarism_assignment = assignment::ActiveModel {
                id: Set(10003),
                module_id: Set(10003),
                name: Set("Plagiarism Assignment".to_string()),
                description: Set(Some("Assignment used to show plagiarism cases".to_string())),
                assignment_type: Set(AssignmentType::Practical),
                status: Set(Status::Setup),
                available_from: Set(Utc::now()),
                due_date: Set(Utc::now() + chrono::Duration::days(7)),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
                ..Default::default()
            };

            let _ = plagiarism_assignment.insert(db).await;

            let gatlam_assignment = assignment::ActiveModel {
                id: Set(10004),
                module_id: Set(10003),
                name: Set("GATLAM Assignment".to_string()),
                description: Set(Some("Assignment used to show GATLAM".to_string())),
                assignment_type: Set(AssignmentType::Practical),
                status: Set(Status::Setup),
                available_from: Set(Utc::now()),
                due_date: Set(Utc::now() + chrono::Duration::days(7)),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
                ..Default::default()
            };

            let _ = gatlam_assignment.insert(db).await;
        })
    }
}
