use crate::seed::Seeder;
use rand::rngs::{OsRng, StdRng};
use rand::{SeedableRng, seq::SliceRandom};
use services::assignment::AssignmentService;
use services::service::{AppError, Service};
use services::ticket::{CreateTicket, TicketService};
use services::user::UserService;
use std::pin::Pin;

pub struct TicketSeeder;

impl Seeder for TicketSeeder {
    fn seed<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
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

            let assignments = AssignmentService::find_all(&vec![], &vec![], None).await?;
            let users = UserService::find_all(&vec![], &vec![], None).await?;

            if assignments.is_empty() || users.is_empty() {
                return Err(AppError::DatabaseUnknown);
            }

            for _ in 0..50 {
                let assignment = assignments.choose(&mut rng).unwrap();
                let user = users.choose(&mut rng).unwrap();

                let title = titles.choose(&mut rng).unwrap().to_string();
                let description = descriptions.choose(&mut rng).unwrap().to_string();
                let status = statuses.choose(&mut rng).unwrap();

                TicketService::create(CreateTicket {
                    assignment_id: assignment.id,
                    user_id: user.id,
                    title: title,
                    description: description,
                    status: status.to_string(),
                })
                .await?;
            }

            Ok(())
        })
    }
}
