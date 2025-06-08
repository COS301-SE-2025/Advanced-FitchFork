use crate::seed::Seeder;
use db::models::{assignment_submission, plagiarism_case};
use sea_orm::{DatabaseConnection, EntityTrait};
use sea_orm::ActiveModelTrait;
use chrono::Utc;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::collections::HashSet;

pub struct PlagiarismCaseSeeder;

#[async_trait::async_trait]
impl Seeder for PlagiarismCaseSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        // Fetch all assignment submissions
        let submissions = assignment_submission::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch assignment submissions");

        if submissions.len() < 2 {
            eprintln!("Not enough submissions to create plagiarism cases");
            return;
        }
        let mut rng = StdRng::from_entropy();
        let mut pairs = HashSet::new();
        let mut attempts = 0;

        // Try to create up to 100 unique cross-user pairs
        while pairs.len() < 100 && attempts < 10000 {
            let sub1 = submissions.choose(&mut rng).unwrap();
            let sub2 = submissions.choose(&mut rng).unwrap();
            attempts += 1;

            if sub1.id == sub2.id || sub1.user_id == sub2.user_id {
                continue; // Skip same submission or same user
            }

            let mut pair = vec![sub1.id, sub2.id];
            pair.sort();

            if pairs.contains(&pair) {
                continue; // Skip duplicates
            }

            pairs.insert(pair.clone());

            let description = format!(
                "Possible plagiarism detected between submission {} and {}",
                pair[0], pair[1]
            );

            let now = Utc::now();
            let case = plagiarism_case::ActiveModel {
                submission_id_1: sea_orm::ActiveValue::Set(pair[0]),
                submission_id_2: sea_orm::ActiveValue::Set(pair[1]),
                description: sea_orm::ActiveValue::Set(description),
                created_at: sea_orm::ActiveValue::Set(now),
                updated_at: sea_orm::ActiveValue::Set(now),
                ..Default::default()
            };

            if let Err(e) = case.insert(db).await {
                eprintln!(
                    "Failed to insert plagiarism case for submissions {} and {}: {}",
                    pair[0], pair[1], e
                );
            }
        }
    }
}
