use crate::seed::Seeder;
use db::models::{achievement_progress::Model as AchievementProgress, achievements, user};
use sea_orm::{DatabaseConnection, EntityTrait, QuerySelect};

pub struct AchievementProgressSeeder;

#[async_trait::async_trait]
impl Seeder for AchievementProgressSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        // Get some existing users and achievements for seeding progress
        let users = user::Entity::find()
            .limit(10)
            .all(db)
            .await
            .unwrap_or_default();

        let achievements = achievements::Entity::find()
            .all(db)
            .await
            .unwrap_or_default();

        if users.is_empty() || achievements.is_empty() {
            println!("Warning: No users or achievements found for progress seeding");
            return;
        }

        // Create some sample progress records
        for (user_index, user) in users.iter().enumerate() {
            for (achievement_index, achievement) in achievements.iter().enumerate() {
                // Create varied progress based on indices to make it interesting
                let should_create_progress = match (user_index + achievement_index) % 4 {
                    0 => true,  // 25% chance
                    1 => false, // Skip
                    2 => user_index % 2 == 0, // 50% chance for even users
                    _ => achievement_index < 3, // First few achievements
                };

                if should_create_progress {
                    let level = match (user_index + achievement_index) % 5 {
                        0 => 0, // Just started
                        1 => 1, // Completed first level
                        2 => 2, // Mid progress
                        3 => achievement.levels.min(3), // Near completion
                        _ => achievement.levels, // Completed
                    };

                    let progress = match level {
                        0 => fastrand::i32(1..50),       // Some initial progress
                        l if l >= achievement.levels => 0, // Completed, no more progress needed
                        _ => fastrand::i32(10..100),     // Mid-level progress
                    };

                    let _ = AchievementProgress::create(
                        db,
                        user.id,
                        achievement.id,
                        level,
                        progress,
                    ).await;
                }
            }
        }

        // Create some specific test progress records
        if let (Some(first_user), Some(first_achievement)) = (users.first(), achievements.first()) {
            // Ensure first user has some progress on first achievement
            let _ = AchievementProgress::update_or_create_progress(
                db,
                first_user.id,
                first_achievement.id,
                1,
                25,
            ).await;
        }

        // Create progress records for "admin" user if it exists
        if let Ok(Some(admin_user)) = user::Model::get_by_username(db, "admin").await {
            for achievement in &achievements[0..3.min(achievements.len())] {
                let _ = AchievementProgress::create(
                    db,
                    admin_user.id,
                    achievement.id,
                    achievement.levels / 2, // Half completed
                    50,
                ).await;
            }
        }

        // Create progress records for "student" user if it exists
        if let Ok(Some(student_user)) = user::Model::get_by_username(db, "student").await {
            for (index, achievement) in achievements.iter().enumerate() {
                if index < 5 { // First 5 achievements
                    let level = if index == 0 { achievement.levels } else { index as i32 % 3 };
                    let progress = if level >= achievement.levels { 0 } else { 30 + index as i32 * 10 };
                    
                    let _ = AchievementProgress::create(
                        db,
                        student_user.id,
                        achievement.id,
                        level,
                        progress,
                    ).await;
                }
            }
        }
    }
}