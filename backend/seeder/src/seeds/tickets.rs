use crate::seed::Seeder;
use chrono::Utc;
use db::models::{tickets, assignment, user};
use rand::rngs::{OsRng, StdRng};
use rand::{seq::SliceRandom, SeedableRng};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::pin::Pin;

pub struct TicketSeeder;

impl Seeder for TicketSeeder {
    fn seed<'a>(&'a self, db: &'a DatabaseConnection) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let mut rng = StdRng::from_rng(OsRng).expect("Failed to seed RNG");

            let statuses = ["open", "closed"];
            let titles = [
                "Issue with assignment",
                "Question about lecture",
                "Bug in code submission",
                "Request for extension",
                "Clarification on module content",
            ];
            let descriptions = [
                "Student reports unexpected behavior.",
                "Need clarification on requirements.",
                "Encountered compilation errors.",
                "Deadline extension requested.",
                "Other module-related inquiry.",
            ];

            let assignments = assignment::Entity::find().all(db).await.unwrap_or_default();
            let users = user::Entity::find().all(db).await.unwrap_or_default();

            if assignments.is_empty() || users.is_empty() {
                return;
            }

            for _ in 0..50 {
                let assignment = assignments.choose(&mut rng).unwrap();
                let user = users.choose(&mut rng).unwrap();

                let title = titles.choose(&mut rng).unwrap().to_string();
                let description = descriptions.choose(&mut rng).unwrap().to_string();
                let status = statuses.choose(&mut rng).unwrap();

                let ticket = tickets::ActiveModel {
                    assignment_id: Set(assignment.id),
                    user_id: Set(user.id),
                    title: Set(title),
                    description: Set(description),
                    status: Set(status.parse().unwrap()),
                    created_at: Set(Utc::now()),
                    updated_at: Set(Utc::now()),
                    ..Default::default()
                };

                let _ = ticket.insert(db).await;
            }
        })
    }
}
