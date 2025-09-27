use crate::seed::Seeder;
use db::models::achievements::Model as Achievement;
use sea_orm::DatabaseConnection;
use serde_json::Value;
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

        let achievements_data: Value = match serde_json::from_str(&achievements_json) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error parsing achievements.json: {}", e);
                return;
            }
        };

        let achievements = match achievements_data.get("achievements") {
            Some(achievements) => achievements,
            None => {
                eprintln!("No 'achievements' key found in achievements.json");
                return;
            }
        };

        // Seed achievements from JSON
        if let Some(achievements_obj) = achievements.as_object() {
            for (condition_id, achievement_data) in achievements_obj {
                if let Some(achievement) = achievement_data.as_object() {
                    let name = achievement.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown Achievement");
                    
                    let description = achievement.get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("No description available");
                    
                    let is_positive = achievement.get("is_positive")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);
                    
                    let levels = achievement.get("levels")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(1) as i32;

                    let _ = Achievement::create(
                        db,
                        name,
                        description,
                        is_positive,
                        levels,
                        condition_id,
                    )
                    .await;
                }
            }
        }

        // Add some additional hardcoded achievements as fallback/examples
        let _ = Achievement::create(
            db,
            "Welcome to FitchFork",
            "Complete your first login to the system",
            true,
            1,
            "first_login",
        )
        .await;

        let _ = Achievement::create(
            db,
            "Assignment Master",
            "Complete all assignments in a module with high scores",
            true,
            1,
            "assignment_master",
        )
        .await;

        let _ = Achievement::create(
            db,
            "Code Reviewer",
            "Review and provide feedback on peer submissions",
            true,
            3,
            "code_reviewer",
        )
        .await;

        let _ = Achievement::create(
            db,
            "Attendance Champion",
            "Maintain perfect attendance for a semester",
            true,
            1,
            "attendance_champion",
        )
        .await;

        let _ = Achievement::create(
            db,
            "No Show",
            "Miss multiple classes without notice",
            false,
            2,
            "no_show",
        )
        .await;

        let _ = Achievement::create(
            db,
            "Copy Cat",
            "Multiple instances of similar code detected",
            false,
            1,
            "copy_cat",
        )
        .await;
    }
}