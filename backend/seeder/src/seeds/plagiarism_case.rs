use crate::seed::Seeder;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use services::assignment_submission::AssignmentSubmissionService;
use services::plagiarism_case::{CreatePlagiarismCase, PlagiarismCaseService};
use services::service::{AppError, Service};
use std::pin::Pin;

pub struct PlagiarismCaseSeeder;

impl Seeder for PlagiarismCaseSeeder {
    fn seed<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            // Fetch all assignment submissions
            let submissions = AssignmentSubmissionService::find_all(&vec![], &vec![], None).await?;

            if submissions.len() < 2 {
                eprintln!("Not enough submissions to create plagiarism cases");
                return Err(AppError::DatabaseUnknown);
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
                return Err(AppError::DatabaseUnknown);
            }

            // Shuffle and limit to 100 pairs
            valid_pairs.shuffle(&mut rng);
            let selected_pairs = valid_pairs.into_iter().take(100);

            for (assignment_id, pair) in selected_pairs {
                if assignment_id != 10003 && assignment_id != 10004 {
                    let description = format!(
                        "Possible plagiarism detected between submission {} and {}",
                        pair[0], pair[1]
                    );

                    PlagiarismCaseService::create(CreatePlagiarismCase {
                        assignment_id: assignment_id,
                        submission_id_1: pair[0],
                        submission_id_2: pair[1],
                        description: description,
                        similarity: 0.0,
                    })
                    .await?;
                }
            }

            Ok(())
        })
    }
}
