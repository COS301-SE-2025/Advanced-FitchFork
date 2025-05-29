use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, Set, DatabaseConnection};
use db::models::{assignment, assignment::AssignmentType, module};
use crate::seed::Seeder;

pub struct AssignmentSeeder;

#[async_trait::async_trait]
impl Seeder for AssignmentSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        let modules = module::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch modules");

        for m in &modules {
            for i in 0..3 {
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
    }
}
