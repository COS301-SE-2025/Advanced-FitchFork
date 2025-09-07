use crate::seed::Seeder;
use chrono::Utc;
use db::models::module;
use rand::rngs::OsRng;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng, seq::SliceRandom};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use std::pin::Pin;

pub struct ModuleSeeder;

impl Seeder for ModuleSeeder {
    fn seed<'a>(&'a self, db: &'a DatabaseConnection) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
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

                let m = module::ActiveModel {
                    code: Set(code),
                    year: Set(rng.gen_range(2020..=2025)),
                    credits: Set(*credit_options.as_slice().choose(&mut rng).unwrap()),
                    description: Set(Some(
                        descriptions
                            .as_slice()
                            .choose(&mut rng)
                            .unwrap()
                            .to_string(),
                    )),
                    created_at: Set(Utc::now()),
                    updated_at: Set(Utc::now()),
                    ..Default::default()
                };

                let _ = m.insert(db).await;
            }

            let special_module = module::ActiveModel {
                id: Set(9999),
                code: Set("TEST9999".to_string()),
                year: Set(2025),
                credits: Set(16),
                description: Set(Some(
                    "Special test module for integration testing".to_string(),
                )),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
                ..Default::default()
            };

            let _ = special_module.insert(db).await;

            let special_module2 = module::ActiveModel {
                id: Set(9998),
                code: Set("TEST9998".to_string()),
                year: Set(2025),
                credits: Set(16),
                description: Set(Some(
                    "Special test module for integration testing".to_string(),
                )),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
                ..Default::default()
            };

        let _ = special_module2.insert(db).await;

        let dem_003 = module::ActiveModel {
            id: Set(10003),
            code: Set("DEM003".to_string()),
            year: Set(2025),
            credits: Set(16),
            description: Set(Some("Module for Demo 3".to_string())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        let _ = dem_003.insert(db).await;
    }
}
