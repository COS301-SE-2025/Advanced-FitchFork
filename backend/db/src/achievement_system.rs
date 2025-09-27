use crate::models::achievements::AchievementDefinition;
use crate::models::achievement_progress::Model as AchievementProgress;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use serde_json::json;

/// Achievement system manager that handles event processing and progress updates
pub struct AchievementSystem {
    /// Loaded achievement definitions from JSON
    pub achievements: HashMap<String, AchievementDefinition>,
}

impl AchievementSystem {
    /// Creates a new achievement system by loading from achievements.json
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let json_content = fs::read_to_string("achievements.json")?;
        let achievements = AchievementDefinition::load_from_json(&json_content)?;
        
        Ok(Self { achievements })
    }

    /// Processes an event and updates user achievement progress
    pub async fn process_event(
        &self,
        db: &DatabaseConnection,
        user_id: i64,
        event_type: &str,
        event_data: &Value,
    ) -> Result<Vec<AchievementUpdate>, sea_orm::DbErr> {
        let mut updates = Vec::new();

        // Check all achievements for matches
        for (achievement_id, definition) in &self.achievements {
            if definition.condition.evaluate(event_type, event_data) {
                // Get database achievement
                let db_achievement = crate::models::achievements::Model::get_by_condition_id(
                    db, achievement_id
                ).await?;

                if let Some(db_achievement) = db_achievement {
                    // Update or create progress
                    let progress = AchievementProgress::update_or_create_progress(
                        db,
                        user_id,
                        db_achievement.id,
                        0, // Don't increment level automatically
                        1, // Increment progress by 1
                    ).await?;

                    // Check if user reached a new level
                    let current_level = progress.current_level as u8;
                    let next_level = current_level + 1;
                    
                    if next_level <= 5 {
                        if definition.is_level_reached(next_level, progress.progress_value as u32) {
                            // User reached new level!
                            let updated_progress = AchievementProgress::update_progress(
                                db,
                                progress.id,
                                next_level as i32,
                                progress.progress_value, // Keep current progress
                            ).await?;

                            updates.push(AchievementUpdate {
                                achievement_id: achievement_id.clone(),
                                achievement_name: definition.name.clone(),
                                user_id,
                                old_level: current_level,
                                new_level: next_level,
                                progress_value: updated_progress.progress_value as u32,
                                is_positive: definition.is_positive,
                            });
                        }
                    }
                }
            }
        }

        Ok(updates)
    }

    /// Gets all achievements a user has progress on
    pub async fn get_user_achievements(
        &self,
        db: &DatabaseConnection,
        user_id: i64,
    ) -> Result<Vec<UserAchievementStatus>, sea_orm::DbErr> {
        let progress_records = AchievementProgress::get_by_user(db, user_id).await?;
        let mut statuses = Vec::new();

        for progress in progress_records {
            // Get achievement from database
            let db_achievement = crate::models::achievements::Entity::find_by_id(progress.achievement_id)
                .one(db)
                .await?;

            if let Some(db_achievement) = db_achievement {
                // Get definition from loaded JSON
                if let Some(definition) = self.achievements.get(&db_achievement.condition_id) {
                    statuses.push(UserAchievementStatus {
                        achievement_id: db_achievement.condition_id.clone(),
                        name: definition.name.clone(),
                        description: definition.description.clone(),
                        is_positive: definition.is_positive,
                        current_level: progress.current_level as u8,
                        progress_value: progress.progress_value as u32,
                        level_thresholds: definition.level_thresholds,
                        is_completed: progress.current_level >= 5,
                        next_threshold: if progress.current_level < 5 {
                            definition.get_level_threshold((progress.current_level + 1) as u8)
                        } else {
                            None
                        },
                    });
                }
            }
        }

        Ok(statuses)
    }

    /// Gets leaderboard for a specific achievement
    pub async fn get_achievement_leaderboard(
        &self,
        db: &DatabaseConnection,
        achievement_id: &str,
        limit: u64,
    ) -> Result<Vec<LeaderboardEntry>, sea_orm::DbErr> {
        // Get database achievement
        let db_achievement = crate::models::achievements::Model::get_by_condition_id(
            db, achievement_id
        ).await?;

        if let Some(db_achievement) = db_achievement {
            let leaderboard = AchievementProgress::get_leaderboard(
                db, db_achievement.id, limit
            ).await?;

            let mut entries = Vec::new();
            for progress in leaderboard {
                // Get user info
                let user = crate::models::user::Entity::find_by_id(progress.user_id)
                    .one(db)
                    .await?;

                if let Some(user) = user {
                    entries.push(LeaderboardEntry {
                        user_id: user.id,
                        username: user.username,
                        current_level: progress.current_level as u8,
                        progress_value: progress.progress_value as u32,
                    });
                }
            }

            Ok(entries)
        } else {
            Ok(Vec::new())
        }
    }
}

/// Represents an achievement level update
#[derive(Debug, Clone)]
pub struct AchievementUpdate {
    pub achievement_id: String,
    pub achievement_name: String,
    pub user_id: i64,
    pub old_level: u8,
    pub new_level: u8,
    pub progress_value: u32,
    pub is_positive: bool,
}

/// Represents a user's status for a specific achievement
#[derive(Debug, Clone)]
pub struct UserAchievementStatus {
    pub achievement_id: String,
    pub name: String,
    pub description: String,
    pub is_positive: bool,
    pub current_level: u8,
    pub progress_value: u32,
    pub level_thresholds: [u32; 5],
    pub is_completed: bool,
    pub next_threshold: Option<u32>,
}

/// Represents a leaderboard entry
#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub user_id: i64,
    pub username: String,
    pub current_level: u8,
    pub progress_value: u32,
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use crate::test_utils::setup_test_db;

    #[tokio::test]
    async fn test_achievement_system_event_processing() {
        // Note: This test would need achievements.json to be available
        // and proper test data setup. This is a demonstration of usage.
        
        let db = setup_test_db().await;
        
        // Create test user
        let user = crate::models::user::Model::create(
            &db, "test_user", "test@example.com", "password", false
        ).await.expect("Failed to create user");

        // This would normally load from achievements.json
        // For testing, we'd need to either mock the file or have test data
        
        // Example event data
        let event_data = json!({
            "score": 100,
            "attempt": 1,
            "assignment_id": 123,
            "submitted_before_deadline": true
        });

        // Process assignment graded event
        // let system = AchievementSystem::new().unwrap();
        // let updates = system.process_event(&db, user.id, "assignment_graded", &event_data).await.unwrap();
        
        // Verify achievements were updated
        // assert!(!updates.is_empty());
    }
}

/// Example usage functions
impl AchievementSystem {
    /// Example: Process an assignment submission
    pub async fn handle_assignment_submitted(
        &self,
        db: &DatabaseConnection,
        user_id: i64,
        assignment_id: i64,
        attempt: u32,
        hours_since_release: f64,
        submitted_before_deadline: bool,
        submission_count: u32,
    ) -> Result<Vec<AchievementUpdate>, sea_orm::DbErr> {
        let event_data = json!({
            "assignment_id": assignment_id,
            "attempt": attempt,
            "hours_since_release": hours_since_release,
            "submitted_before_deadline": submitted_before_deadline,
            "submission_count": submission_count,
        });

        self.process_event(db, user_id, "assignment_submitted", &event_data).await
    }

    /// Example: Process an assignment being graded
    pub async fn handle_assignment_graded(
        &self,
        db: &DatabaseConnection,
        user_id: i64,
        assignment_id: i64,
        score: f64,
        max_score: f64,
        attempt: u32,
    ) -> Result<Vec<AchievementUpdate>, sea_orm::DbErr> {
        let event_data = json!({
            "assignment_id": assignment_id,
            "score": score,
            "max_score": max_score,
            "attempt": attempt,
        });

        self.process_event(db, user_id, "assignment_graded", &event_data).await
    }

    /// Example: Process attendance recording
    pub async fn handle_attendance_recorded(
        &self,
        db: &DatabaseConnection,
        user_id: i64,
        session_id: i64,
        attended: bool,
        excused: bool,
    ) -> Result<Vec<AchievementUpdate>, sea_orm::DbErr> {
        let event_data = json!({
            "session_id": session_id,
            "attended": attended,
            "excused": excused,
        });

        self.process_event(db, user_id, "attendance_recorded", &event_data).await
    }
}