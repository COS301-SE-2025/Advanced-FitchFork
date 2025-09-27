use crate::seed::Seeder;
use db::models::achievements::{Model as Achievement, AchievementDefinition};
use sea_orm::DatabaseConnection;
use std::fs;

pub struct AchievementSeeder;

#[async_trait::async_trait]
impl Seeder for AchievementSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        // Read achievements from JSON file
        let achievements_json = match fs::read_to_string("achievements.json") {
            Ok(content) => content,
            Err(_) => {
                eprintln!("Warning: achievements.json file not found, seeding default achievements");
                return;
            }
        };

        // Parse using the new schema structs
        let achievement_definitions = match AchievementDefinition::load_from_json(&achievements_json) {
            Ok(definitions) => definitions,
            Err(e) => {
                eprintln!("Error parsing achievements.json: {}", e);
                return;
            }
        };

        // Seed achievements from JSON using the new schema
        for (condition_id, definition) in achievement_definitions {
            let _ = Achievement::create(
                db,
                &definition.name,
                &definition.description,
                definition.is_positive,
                5, // All achievements now have 5 levels
                &condition_id,
            )
            .await;
        }

        // Add some additional hardcoded achievements as fallback/examples
        let _ = Achievement::create(
            db,
            "Welcome to FitchFork",
            "Complete your first login to the system",
            true,
            5,
            "first_login",
        )
        .await;

        let _ = Achievement::create(
            db,
            "Assignment Master",
            "Complete all assignments in a module with high scores",
            true,
            5,
            "assignment_master",
        )
        .await;

        let _ = Achievement::create(
            db,
            "Code Reviewer",
            "Review and provide feedback on peer submissions",
            true,
            5,
            "code_reviewer",
        )
        .await;
    }
}