use crate::seed::Seeder;
use chrono::Utc;
use db::models::{assignment, assignment::AssignmentType, module};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

pub struct AssignmentSeeder;

#[async_trait::async_trait]
impl Seeder for AssignmentSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        let modules = module::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch modules");

        for m in &modules {
            if m.id == 9999 {
                continue;
            }
            for i in 0..2 {
                let a = assignment::ActiveModel {
                    module_id: Set(m.id),
                    name: Set(format!("Assignment {i}")),
                    description: Set(Some("Auto seeded".to_string())),
                    assignment_type: Set(AssignmentType::Practical),
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
            available_from: Set(Utc::now()),
            due_date: Set(Utc::now() + chrono::Duration::days(7)),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        let _ = special_assignment.insert(db).await;
    }
}
