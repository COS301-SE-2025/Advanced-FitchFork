use crate::seed::Seeder;
use chrono::Utc;
use db::models::{assignment, assignment_submission, user};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

pub struct AssignmentSubmissionSeeder;

#[async_trait::async_trait]
impl Seeder for AssignmentSubmissionSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        let assignments = assignment::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch assignments");

        let users = user::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch users");

        if users.is_empty() {
            panic!("No users found — at least one user must exist to seed assignment_submissions");
        }

        for a in &assignments {
            for u in &users {
                let submission = assignment_submission::ActiveModel {
                    assignment_id: Set(a.id),
                    user_id: Set(u.id),
                    created_at: Set(Utc::now()),
                    updated_at: Set(Utc::now()),
                    ..Default::default()
                };

                let _ = submission.insert(db).await;
            }
        }
    }
}
