use crate::seed::Seeder;
use chrono::Utc;
use db::models::{assignment_submission, plagiarism_case};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};

pub struct PlagiarismCaseSeeder;

#[async_trait::async_trait]
impl Seeder for PlagiarismCaseSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        // Fetch all assignment submissions
        let submissions = match assignment_submission::Entity::find().all(db).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to fetch assignment submissions: {}", e);
                return;
            }
        };

        if submissions.len() < 2 {
            eprintln!("Not enough submissions to create plagiarism cases");
            return;
        }

        let mut rng = StdRng::from_entropy();

        // Generate all valid unique pairs of submissions with different users and same assignment
        let mut valid_pairs = Vec::new();
        for i in 0..submissions.len() {
            for j in (i + 1)..submissions.len() {
                let sub1 = &submissions[i];
                let sub2 = &submissions[j];
                if sub1.user_id != sub2.user_id && sub1.assignment_id == sub2.assignment_id {
                    let mut pair = vec![sub1.id, sub2.id];
                    pair.sort();
                    // Store assignment_id along with the pair
                    valid_pairs.push((sub1.assignment_id, pair));
                }
            }
        }

        if valid_pairs.is_empty() {
            eprintln!("No valid cross-user submission pairs found for the same assignment");
            return;
        }

        // Shuffle and limit to 100 pairs
        valid_pairs.shuffle(&mut rng);
        let selected_pairs = valid_pairs.into_iter().take(100);

        let now = Utc::now();

        for (assignment_id, pair) in selected_pairs {
            if assignment_id != 10003 && assignment_id != 10004 {
                let description = format!(
                    "Possible plagiarism detected between submission {} and {}",
                    pair[0], pair[1]
                );

                let case = plagiarism_case::ActiveModel {
                    assignment_id: sea_orm::ActiveValue::Set(assignment_id),
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
}
