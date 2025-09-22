use crate::seed::Seeder;
use rand::rngs::OsRng;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng, seq::SliceRandom};
use services::module::{CreateModule, ModuleService};
use services::service::{AppError, Service};
use std::pin::Pin;

pub struct ModuleSeeder;

impl Seeder for ModuleSeeder {
    fn seed<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            // Use a Send-compatible RNG
            let mut rng = StdRng::from_rng(OsRng).expect("Failed to seed RNG");

            let credit_options = [8, 16, 24];
            let descriptions = [
                "Advanced Algorithms",
                "Distributed Systems",
                "Computer Graphics",
                "Operating Systems",
                "AI Fundamentals",
                "Software Engineering",
                "Cybersecurity Basics",
                "Functional Programming",
                "Compiler Construction",
                "Mobile Development",
            ];

            for _ in 0..10 {
                let range_choice = rng.gen_range(0..3);
                let code_number = match range_choice {
                    0 => rng.gen_range(100..200),
                    1 => rng.gen_range(200..300),
                    _ => rng.gen_range(300..400),
                };
                let code = format!("COS{}", code_number);

                ModuleService::create(CreateModule {
                    id: None,
                    code: code,
                    year: rng.gen_range(2020..=2025),
                    credits: *credit_options.as_slice().choose(&mut rng).unwrap(),
                    description: Some(
                        descriptions
                            .as_slice()
                            .choose(&mut rng)
                            .unwrap()
                            .to_string(),
                    ),
                })
                .await?;
            }

            ModuleService::create(CreateModule {
                id: Some(9999),
                code: "TEST9999".to_string(),
                year: 2025,
                credits: 16,
                description: Some("Special test module for integration testing".to_string()),
            })
            .await?;

            ModuleService::create(CreateModule {
                id: Some(9998),
                code: "TEST9998".to_string(),
                year: 2025,
                credits: 16,
                description: Some("Special test module for integration testing".to_string()),
            })
            .await?;

            ModuleService::create(CreateModule {
                id: Some(10003),
                code: "DEM003".to_string(),
                year: 2025,
                credits: 16,
                description: Some("Module for Demo 3".to_string()),
            })
            .await?;

            Ok(())
        })
    }
}
