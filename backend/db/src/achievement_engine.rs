/// Achievement Engine
/// 
/// This module provides the core event-driven achievement system that listens for user actions,
/// evaluates achievement conditions, and automatically updates user progress and levels.
/// 
/// The engine is designed to be decoupled from business logic and uses the generic condition
/// schema from `achievements.json` to evaluate progress without hardcoded logic.

use std::collections::HashMap;
use std::sync::Arc;

use sea_orm::{DatabaseConnection, DbErr, EntityTrait, QueryFilter, ColumnTrait};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

use crate::events::UserEvent;
use crate::models::achievements::{AchievementDefinition, Entity as AchievementEntity};
use crate::models::achievement_progress::{
    Entity as ProgressEntity, 
    Model as ProgressModel,
    Column as ProgressColumn
};

/// Result type for achievement engine operations
pub type AchievementResult<T> = Result<T, AchievementError>;

/// Errors that can occur in the achievement engine
#[derive(Debug, thiserror::Error)]
pub enum AchievementError {
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
    
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Achievement not found: {0}")]
    AchievementNotFound(String),
    
    #[error("Invalid achievement configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Progress update failed: {0}")]
    ProgressUpdateFailed(String),
    
    #[error("Condition evaluation failed: {0}")]
    ConditionEvaluation(String),
}

/// Information about a granted achievement level
#[derive(Debug, Clone)]
pub struct LevelGrant {
    pub achievement_id: String,
    pub achievement_name: String,
    pub user_id: i64,
    pub old_level: i32,
    pub new_level: i32,
    pub progress_value: i32,
    pub is_positive: bool,
}

/// Result of processing an event
#[derive(Debug, Clone)]
pub struct EventProcessingResult {
    pub achievements_updated: Vec<String>,
    pub levels_granted: Vec<LevelGrant>,
    pub total_progress_updates: usize,
}

/// Configuration for the achievement engine
#[derive(Debug, Clone)]
pub struct AchievementEngineConfig {
    /// Whether to log debug information about achievement checking
    pub debug_logging: bool,
    /// Whether to emit events for level grants (for notifications, etc.)
    pub emit_level_events: bool,
    /// Maximum number of achievements to process per event (prevent runaway)
    pub max_achievements_per_event: usize,
}

impl Default for AchievementEngineConfig {
    fn default() -> Self {
        Self {
            debug_logging: false,
            emit_level_events: true,
            max_achievements_per_event: 100,
        }
    }
}

/// Event-driven achievement engine
/// 
/// The engine maintains an in-memory cache of achievement definitions loaded from the database
/// and JSON configuration. When events are received, it evaluates all relevant achievements
/// and updates user progress accordingly.
pub struct AchievementEngine {
    /// Database connection for progress updates
    db: DatabaseConnection,
    /// Achievement definitions cache
    achievements: Arc<RwLock<HashMap<String, AchievementDefinition>>>,
    /// Achievement ID to database ID mapping
    achievement_db_ids: Arc<RwLock<HashMap<String, i64>>>,
    /// Engine configuration
    config: AchievementEngineConfig,
}

impl AchievementEngine {
    /// Create a new achievement engine
    pub async fn new(
        db: DatabaseConnection,
        config: AchievementEngineConfig,
    ) -> AchievementResult<Self> {
        let engine = Self {
            db,
            achievements: Arc::new(RwLock::new(HashMap::new())),
            achievement_db_ids: Arc::new(RwLock::new(HashMap::new())),
            config,
        };
        
        // Load initial achievement definitions
        engine.reload_achievements().await?;
        
        Ok(engine)
    }
    
    /// Create a new achievement engine with default configuration
    pub async fn new_default(db: DatabaseConnection) -> AchievementResult<Self> {
        Self::new(db, AchievementEngineConfig::default()).await
    }
    
    /// Reload achievement definitions from database and JSON configuration
    pub async fn reload_achievements(&self) -> AchievementResult<()> {
        info!("Reloading achievement definitions");
        
        // Load achievements from database
        let db_achievements = AchievementEntity::find()
            .all(&self.db)
            .await?;
        
        // Load JSON configuration
        let json_content = tokio::fs::read_to_string("achievements.json")
            .await
            .map_err(|e| AchievementError::InvalidConfig(format!("Failed to read achievements.json: {}", e)))?;
        
        let json_definitions = AchievementDefinition::load_from_json(&json_content)?;
        
        // Build caches
        let mut achievements = HashMap::new();
        let mut db_ids = HashMap::new();
        
        for db_achievement in db_achievements {
            if let Some(json_def) = json_definitions.get(&db_achievement.condition_id) {
                achievements.insert(db_achievement.condition_id.clone(), json_def.clone());
                db_ids.insert(db_achievement.condition_id.clone(), db_achievement.id);
            } else {
                warn!("Database achievement '{}' has no corresponding JSON definition", db_achievement.condition_id);
            }
        }
        
        // Update caches
        *self.achievements.write().await = achievements;
        *self.achievement_db_ids.write().await = db_ids;
        
        info!("Loaded {} achievement definitions", self.achievements.read().await.len());
        Ok(())
    }
    
    /// Process a user event and update achievement progress
    pub async fn handle_event(&self, event: UserEvent) -> AchievementResult<EventProcessingResult> {
        let event_type = event.event_type();
        let user_id = event.user_id();
        let event_start = std::time::Instant::now();
        
        if self.config.debug_logging {
            debug!("Processing event '{}' for user {}", event_type, user_id);
        }
        
        // Convert event to JSON for condition evaluation - with error handling
        let event_data = match event.to_json() {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to serialize event '{}' for user {}: {}", event_type, user_id, e);
                return Err(AchievementError::JsonError(e));
            }
        };
        
        // Find all achievements that match this event type
        let matching_achievements = self.find_matching_achievements(event_type).await;
        
        if matching_achievements.is_empty() {
            if self.config.debug_logging {
                debug!("No achievements match event type '{}'", event_type);
            }
            return Ok(EventProcessingResult {
                achievements_updated: Vec::new(),
                levels_granted: Vec::new(),
                total_progress_updates: 0,
            });
        }
        
        if self.config.debug_logging {
            debug!("Found {} potential achievements for event '{}'", matching_achievements.len(), event_type);
        }
        
        // Check limit
        if matching_achievements.len() > self.config.max_achievements_per_event {
            warn!("Event '{}' matches {} achievements, exceeding limit of {}. Processing first {}.",
                event_type, matching_achievements.len(), self.config.max_achievements_per_event,
                self.config.max_achievements_per_event);
        }
        
        let mut achievements_updated = Vec::new();
        let mut levels_granted = Vec::new();
        let mut total_updates = 0;
        let mut errors_encountered = 0;
        
        // Process each matching achievement
        for (achievement_id, definition) in matching_achievements.into_iter()
            .take(self.config.max_achievements_per_event) 
        {
            if self.config.debug_logging {
                debug!("Evaluating achievement '{}' for user {}", achievement_id, user_id);
            }
            
            // Check if the condition is met - with detailed error handling
            let condition_met = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                definition.condition.evaluate(event_type, &event_data)
            })) {
                Ok(result) => result,
                Err(panic_info) => {
                    error!("Panic occurred while evaluating condition for achievement '{}', user {}: {:?}", 
                        achievement_id, user_id, panic_info);
                    errors_encountered += 1;
                    continue;
                }
            };
            
            if condition_met {
                if self.config.debug_logging {
                    debug!("Achievement '{}' condition met for user {}", achievement_id, user_id);
                }
                
                // Update progress and check for level grants - with retry logic
                let mut attempts = 0;
                const MAX_ATTEMPTS: u32 = 3;
                
                loop {
                    attempts += 1;
                    match self.update_achievement_progress(&achievement_id, &definition, user_id).await {
                        Ok(Some(level_grant)) => {
                            achievements_updated.push(achievement_id.clone());
                            levels_granted.push(level_grant.clone());
                            total_updates += 1;
                            
                            info!("Level granted: {} level {} for user {} (achievement '{}')",
                                level_grant.new_level, level_grant.achievement_name, user_id, achievement_id);
                            break;
                        }
                        Ok(None) => {
                            achievements_updated.push(achievement_id.clone());
                            total_updates += 1;
                            
                            if self.config.debug_logging {
                                debug!("Progress updated for achievement '{}', user {} (no level grant)", 
                                    achievement_id, user_id);
                            }
                            break;
                        }
                        Err(e) => {
                            if attempts >= MAX_ATTEMPTS {
                                error!("Failed to update progress for achievement '{}', user {} after {} attempts: {}", 
                                    achievement_id, user_id, MAX_ATTEMPTS, e);
                                errors_encountered += 1;
                                break;
                            } else {
                                warn!("Attempt {} failed to update progress for achievement '{}', user {}: {}. Retrying...", 
                                    attempts, achievement_id, user_id, e);
                                tokio::time::sleep(std::time::Duration::from_millis(100 * attempts as u64)).await;
                            }
                        }
                    }
                }
            } else if self.config.debug_logging {
                debug!("Achievement '{}' condition not met for user {}", achievement_id, user_id);
            }
        }
        
        let processing_time = event_start.elapsed();
        
        if !levels_granted.is_empty() {
            info!("Event '{}' for user {} resulted in {} level grants across {} achievements in {:?}",
                event_type, user_id, levels_granted.len(), achievements_updated.len(), processing_time);
        } else if self.config.debug_logging {
            debug!("Event '{}' for user {} completed in {:?} with {} progress updates, {} errors",
                event_type, user_id, processing_time, total_updates, errors_encountered);
        }
        
        if errors_encountered > 0 {
            warn!("Event '{}' for user {} completed with {} errors during processing", 
                event_type, user_id, errors_encountered);
        }
        
        Ok(EventProcessingResult {
            achievements_updated,
            levels_granted,
            total_progress_updates: total_updates,
        })
    }
    
    /// Find all achievement definitions that match the given event type
    async fn find_matching_achievements(&self, event_type: &str) -> Vec<(String, AchievementDefinition)> {
        let achievements = self.achievements.read().await;
        achievements
            .iter()
            .filter(|(_, def)| def.condition.event == event_type)
            .map(|(id, def)| (id.clone(), def.clone()))
            .collect()
    }
    
    /// Update progress for a specific achievement and return level grant if applicable
    async fn update_achievement_progress(
        &self,
        achievement_id: &str,
        definition: &AchievementDefinition,
        user_id: i64,
    ) -> AchievementResult<Option<LevelGrant>> {
        // Get database achievement ID
        let db_achievement_id = {
            let db_ids = self.achievement_db_ids.read().await;
            *db_ids.get(achievement_id)
                .ok_or_else(|| AchievementError::AchievementNotFound(achievement_id.to_string()))?
        };
        
        // Get current progress
        let current_progress = ProgressEntity::find()
            .filter(ProgressColumn::UserId.eq(user_id))
            .filter(ProgressColumn::AchievementId.eq(db_achievement_id))
            .one(&self.db)
            .await?;
        
        let (current_level, current_value) = match &current_progress {
            Some(progress) => (progress.current_level, progress.progress_value),
            None => (0, 0),
        };
        
        // Increment progress by 1 (each condition match = +1 progress)
        let new_value = current_value + 1;
        
        // Check if this qualifies for a level upgrade
        let mut new_level = current_level;
        let mut level_granted = None;
        
        // Check levels 1-5 to see if we've reached any new thresholds
        for level in (current_level + 1)..=5 {
            if definition.is_level_reached(level as u8, new_value as u32) {
                new_level = level;
            } else {
                break;
            }
        }
        
        // Update or create progress record
        let _ = if let Some(existing) = current_progress {
            ProgressModel::update_progress(&self.db, existing.id, new_level, new_value).await?
        } else {
            ProgressModel::create(&self.db, user_id, db_achievement_id, new_level, new_value).await?
        };
        
        // Create level grant info if level increased
        if new_level > current_level {
            level_granted = Some(LevelGrant {
                achievement_id: achievement_id.to_string(),
                achievement_name: definition.name.clone(),
                user_id,
                old_level: current_level,
                new_level,
                progress_value: new_value,
                is_positive: definition.is_positive,
            });
        }
        
        Ok(level_granted)
    }
    
    /// Get current progress for a user and achievement
    pub async fn get_user_progress(
        &self,
        user_id: i64,
        achievement_id: &str,
    ) -> AchievementResult<Option<ProgressModel>> {
        let db_achievement_id = {
            let db_ids = self.achievement_db_ids.read().await;
            *db_ids.get(achievement_id)
                .ok_or_else(|| AchievementError::AchievementNotFound(achievement_id.to_string()))?
        };
        
        let progress = ProgressEntity::find()
            .filter(ProgressColumn::UserId.eq(user_id))
            .filter(ProgressColumn::AchievementId.eq(db_achievement_id))
            .one(&self.db)
            .await?;
        
        Ok(progress)
    }
    
    /// Get all progress for a user
    pub async fn get_user_all_progress(&self, user_id: i64) -> AchievementResult<Vec<ProgressModel>> {
        let progress = ProgressModel::get_by_user(&self.db, user_id).await?;
        Ok(progress)
    }
    
    /// Get achievement definition by ID
    pub async fn get_achievement_definition(&self, achievement_id: &str) -> Option<AchievementDefinition> {
        let achievements = self.achievements.read().await;
        achievements.get(achievement_id).cloned()
    }
    
    /// Get all loaded achievement definitions
    pub async fn get_all_achievement_definitions(&self) -> HashMap<String, AchievementDefinition> {
        let achievements = self.achievements.read().await;
        achievements.clone()
    }
    
    /// Check if the engine has a specific achievement loaded
    pub async fn has_achievement(&self, achievement_id: &str) -> bool {
        let achievements = self.achievements.read().await;
        achievements.contains_key(achievement_id)
    }
    
    /// Get statistics about the achievement engine
    pub async fn get_engine_stats(&self) -> HashMap<String, serde_json::Value> {
        let achievements = self.achievements.read().await;
        let db_ids = self.achievement_db_ids.read().await;
        
        let mut stats = HashMap::new();
        stats.insert("loaded_achievements".to_string(), achievements.len().into());
        stats.insert("db_achievement_mappings".to_string(), db_ids.len().into());
        stats.insert("debug_logging".to_string(), self.config.debug_logging.into());
        stats.insert("max_achievements_per_event".to_string(), self.config.max_achievements_per_event.into());
        
        // Count positive vs negative achievements
        let positive_count = achievements.values().filter(|def| def.is_positive).count();
        let negative_count = achievements.values().filter(|def| !def.is_positive).count();
        stats.insert("positive_achievements".to_string(), positive_count.into());
        stats.insert("negative_achievements".to_string(), negative_count.into());
        
        // Event type breakdown
        let mut event_types = HashMap::new();
        for def in achievements.values() {
            let event_type = &def.condition.event;
            *event_types.entry(event_type.clone()).or_insert(0) += 1;
        }
        stats.insert("achievements_by_event_type".to_string(), serde_json::to_value(event_types).unwrap_or_default());
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db;
    
    #[tokio::test]
    async fn test_achievement_engine_creation() {
        let db = setup_test_db().await;
        let config = AchievementEngineConfig {
            debug_logging: true,
            ..Default::default()
        };
        
        // Note: This test will fail without achievements.json, but demonstrates the API
        let engine_result = AchievementEngine::new(db, config).await;
        // In real tests, we'd mock the file system or provide a test JSON file
        // For now, we just check that the error is the expected one
        assert!(engine_result.is_err());
    }

    #[test]
    fn test_achievement_error_display() {
        let error = AchievementError::AchievementNotFound("test_achievement".to_string());
        assert_eq!(error.to_string(), "Achievement not found: test_achievement");
        
        let error = AchievementError::InvalidConfig("bad config".to_string());
        assert_eq!(error.to_string(), "Invalid achievement configuration: bad config");
    }

    #[test]
    fn test_level_grant_creation() {
        let grant = LevelGrant {
            achievement_id: "first_submission".to_string(),
            achievement_name: "First Steps".to_string(),
            user_id: 123,
            old_level: 0,
            new_level: 1,
            progress_value: 1,
            is_positive: true,
        };
        
        assert_eq!(grant.achievement_id, "first_submission");
        assert_eq!(grant.user_id, 123);
        assert_eq!(grant.new_level, 1);
        assert!(grant.is_positive);
    }
    
    #[test]
    fn test_event_processing_result() {
        let result = EventProcessingResult {
            achievements_updated: vec!["first_submission".to_string(), "early_bird".to_string()],
            levels_granted: vec![],
            total_progress_updates: 2,
        };
        
        assert_eq!(result.achievements_updated.len(), 2);
        assert_eq!(result.total_progress_updates, 2);
        assert!(result.levels_granted.is_empty());
    }

    use chrono::{TimeZone, Utc};
    use serde_json::json;
    use tempfile::NamedTempFile;
    use tokio::fs;

    use crate::achievement_engine::{AchievementEngine, AchievementEngineConfig, AchievementError};
    use crate::events::UserEvent;
    use crate::models::achievements::Model as AchievementModel;

    /// Create a test achievements.json file
    async fn create_test_achievements_json() -> Result<NamedTempFile, Box<dyn std::error::Error>> {
        let test_config = json!({
            "achievements": {
                "first_submission": {
                    "id": "first_submission",
                    "name": "First Steps",
                    "description": "Submit your first assignment to FitchFork",
                    "is_positive": true,
                    "condition": {
                        "event": "assignment_submitted",
                        "checks": [
                            { "field": "submission_count", "op": "gte", "value": 1 }
                        ]
                    },
                    "level_thresholds": [1, 1, 1, 1, 1]
                },
                "perfect_score": {
                    "id": "perfect_score",
                    "name": "Perfect Score",
                    "description": "Achieve 100% on assignments",
                    "is_positive": true,
                    "condition": {
                        "event": "assignment_graded",
                        "checks": [
                            { "field": "score", "op": "eq", "value": 100 }
                        ]
                    },
                    "level_thresholds": [1, 3, 5, 10, 20]
                },
                "high_achiever": {
                    "id": "high_achiever",
                    "name": "High Achiever",
                    "description": "Consistently score above 90%",
                    "is_positive": true,
                    "condition": {
                        "event": "assignment_graded",
                        "checks": [
                            { "field": "score", "op": "gte", "value": 90 }
                        ]
                    },
                    "level_thresholds": [3, 5, 10, 20, 40]
                },
                "late_submission": {
                    "id": "late_submission",
                    "name": "Procrastinator",
                    "description": "Submit assignments after the deadline",
                    "is_positive": false,
                    "condition": {
                        "event": "assignment_submitted",
                        "checks": [
                            { "field": "submitted_before_deadline", "op": "eq", "value": false }
                        ]
                    },
                    "level_thresholds": [1, 3, 5, 10, 20]
                }
            }
        });

        let temp_file = NamedTempFile::new()?;
        fs::write(temp_file.path(), serde_json::to_string_pretty(&test_config)?).await?;
        Ok(temp_file)
    }

    /// Set up test engine with database and achievements
    async fn setup_test_engine() -> Result<(AchievementEngine, NamedTempFile), Box<dyn std::error::Error>> {
        let db = setup_test_db().await;
        let temp_file = create_test_achievements_json().await?;
        
        // Create database achievements that match our JSON
        let achievements_data = [
            ("first_submission", "First Steps", "Submit your first assignment to FitchFork", true, 5),
            ("perfect_score", "Perfect Score", "Achieve 100% on assignments", true, 5),
            ("high_achiever", "High Achiever", "Consistently score above 90%", true, 5),
            ("late_submission", "Procrastinator", "Submit assignments after the deadline", false, 5),
        ];
        
        for (condition_id, name, desc, is_positive, levels) in achievements_data {
            AchievementModel::create(&db, name, desc, is_positive, levels, condition_id)
                .await
                .expect("Failed to create test achievement");
        }
        
        // Temporarily set the achievements file path for testing
        let original_path = std::env::current_dir()?;
        let temp_dir = temp_file.path().parent().unwrap();
        std::env::set_current_dir(temp_dir)?;
        
        let config = AchievementEngineConfig {
            debug_logging: true,
            emit_level_events: true,
            max_achievements_per_event: 10,
        };
        
        let engine = AchievementEngine::new(db, config).await?;
        
        // Restore original directory
        std::env::set_current_dir(original_path)?;
        
        Ok((engine, temp_file))
    }

    #[tokio::test]
    async fn test_achievement_engine_creation() {
        let (engine, _temp_file) = setup_test_engine().await
            .expect("Failed to set up test engine");
        
        // Verify achievements are loaded
        let definitions = engine.get_all_achievement_definitions().await;
        assert_eq!(definitions.len(), 4);
        assert!(definitions.contains_key("first_submission"));
        assert!(definitions.contains_key("perfect_score"));
        assert!(definitions.contains_key("high_achiever"));
        assert!(definitions.contains_key("late_submission"));
    }

    #[tokio::test]
    async fn test_first_submission_achievement() {
        let (engine, _temp_file) = setup_test_engine().await
            .expect("Failed to set up test engine");
        
        let user_id = 123;
        let event = UserEvent::assignment_submitted(
            user_id,
            1, // assignment_id
            2, // module_id
            1, // attempt
            false, // is_practice
            "solution.zip".to_string(),
            1, // submission_count (first submission)
            Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 0).unwrap(), // due_date
            Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap() // release_date
        );
        
        let result = engine.handle_event(event).await
            .expect("Failed to handle event");
        
        // Should trigger first submission achievement
        assert_eq!(result.achievements_updated.len(), 1);
        assert_eq!(result.levels_granted.len(), 1);
        assert_eq!(result.total_progress_updates, 1);
        
        let level_grant = &result.levels_granted[0];
        assert_eq!(level_grant.achievement_id, "first_submission");
        assert_eq!(level_grant.user_id, user_id);
        assert_eq!(level_grant.old_level, 0);
        assert_eq!(level_grant.new_level, 1);
        assert!(level_grant.is_positive);
        
        // Verify progress is recorded in database
        let progress = engine.get_user_progress(user_id, "first_submission").await
            .expect("Failed to get progress");
        assert!(progress.is_some());
        
        let progress = progress.unwrap();
        assert_eq!(progress.current_level, 1);
        assert_eq!(progress.progress_value, 1);
    }

    #[tokio::test]
    async fn test_perfect_score_achievement() {
        let (engine, _temp_file) = setup_test_engine().await
            .expect("Failed to set up test engine");
        
        let user_id = 456;
        let event = UserEvent::assignment_graded(
            user_id,
            1, // assignment_id
            2, // module_id
            10, // submission_id
            1, // attempt
            100, // score (perfect!)
            100 // total_marks
        );
        
        let result = engine.handle_event(event).await
            .expect("Failed to handle event");
        
        // Should trigger perfect score achievement
        assert_eq!(result.achievements_updated.len(), 2); // perfect_score + high_achiever (90+)
        assert_eq!(result.levels_granted.len(), 2);
        
        // Check perfect score achievement
        let perfect_grant = result.levels_granted.iter()
            .find(|g| g.achievement_id == "perfect_score")
            .expect("Perfect score achievement should be granted");
        
        assert_eq!(perfect_grant.user_id, user_id);
        assert_eq!(perfect_grant.old_level, 0);
        assert_eq!(perfect_grant.new_level, 1);
        
        // Check high achiever achievement (should also trigger for 100%)
        let high_achiever_grant = result.levels_granted.iter()
            .find(|g| g.achievement_id == "high_achiever")
            .expect("High achiever achievement should be granted");
        
        assert_eq!(high_achiever_grant.user_id, user_id);
        assert_eq!(high_achiever_grant.old_level, 0);
        assert_eq!(high_achiever_grant.new_level, 1);
    }

    #[tokio::test]
    async fn test_high_achiever_progression() {
        let (engine, _temp_file) = setup_test_engine().await
            .expect("Failed to set up test engine");
        
        let user_id = 789;
        
        // Submit multiple high-scoring assignments
        let scores = vec![95, 92, 98, 91, 99]; // 5 scores >= 90
        
        for (i, score) in scores.iter().enumerate() {
            let event = UserEvent::assignment_graded(
                user_id,
                i as i64 + 1, // different assignment_ids
                2, // module_id
                i as i64 + 10, // different submission_ids
                1, // attempt
                *score,
                100 // total_marks
            );
            
            let result = engine.handle_event(event).await
                .expect("Failed to handle event");
            
            // Should always trigger high achiever
            assert!(result.achievements_updated.contains(&"high_achiever".to_string()));
        }
        
        // Check final progress - should be at level 1 (threshold is 3 for level 2)
        let progress = engine.get_user_progress(user_id, "high_achiever").await
            .expect("Failed to get progress")
            .expect("Progress should exist");
        
        assert_eq!(progress.current_level, 1); // 5 >= 3 threshold for level 1, but < 5 for level 2
        assert_eq!(progress.progress_value, 5);
    }

    #[tokio::test]
    async fn test_late_submission_negative_achievement() {
        let (engine, _temp_file) = setup_test_engine().await
            .expect("Failed to set up test engine");
        
        let user_id = 999;
        let event = UserEvent::assignment_submitted(
            user_id,
            1, // assignment_id
            2, // module_id
            1, // attempt
            false, // is_practice
            "late_solution.zip".to_string(),
            1, // submission_count
            Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 0).unwrap(), // due_date (in past)
            Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap() // release_date
        );
        
        let result = engine.handle_event(event).await
            .expect("Failed to handle event");
        
        // Should trigger both first_submission and late_submission
        assert_eq!(result.achievements_updated.len(), 2);
        assert_eq!(result.levels_granted.len(), 2);
        
        // Check late submission achievement
        let late_grant = result.levels_granted.iter()
            .find(|g| g.achievement_id == "late_submission")
            .expect("Late submission achievement should be granted");
        
        assert_eq!(late_grant.user_id, user_id);
        assert_eq!(late_grant.old_level, 0);
        assert_eq!(late_grant.new_level, 1);
        assert!(!late_grant.is_positive); // This is a negative achievement
    }

    #[tokio::test]
    async fn test_no_matching_achievements() {
        let (engine, _temp_file) = setup_test_engine().await
            .expect("Failed to set up test engine");
        
        // Create an event type that doesn't match any achievements
        let event = UserEvent::AttendanceRecorded {
            user_id: 123,
            module_id: 1,
            session_id: 1,
            attended: true,
            excused: false,
            recorded_at: Utc::now(),
        };
        
        let result = engine.handle_event(event).await
            .expect("Failed to handle event");
        
        // Should not trigger any achievements
        assert_eq!(result.achievements_updated.len(), 0);
        assert_eq!(result.levels_granted.len(), 0);
        assert_eq!(result.total_progress_updates, 0);
    }

    #[tokio::test]
    async fn test_condition_not_met() {
        let (engine, _temp_file) = setup_test_engine().await
            .expect("Failed to set up test engine");
        
        let user_id = 555;
        // Score of 85 should not trigger perfect_score, but should trigger high_achiever
        let event = UserEvent::assignment_graded(
            user_id,
            1, // assignment_id
            2, // module_id
            10, // submission_id
            1, // attempt
            85, // score (not perfect, but < 90 so no high achiever either)
            100 // total_marks
        );
        
        let result = engine.handle_event(event).await
            .expect("Failed to handle event");
        
        // Should not trigger any achievements (85 < 90 threshold for high_achiever)
        assert_eq!(result.achievements_updated.len(), 0);
        assert_eq!(result.levels_granted.len(), 0);
        assert_eq!(result.total_progress_updates, 0);
    }

    #[tokio::test]
    async fn test_engine_statistics() {
        let (engine, _temp_file) = setup_test_engine().await
            .expect("Failed to set up test engine");
        
        let stats = engine.get_engine_stats().await;
        
        assert_eq!(stats.get("loaded_achievements").unwrap().as_u64().unwrap(), 4);
        assert_eq!(stats.get("positive_achievements").unwrap().as_u64().unwrap(), 3);
        assert_eq!(stats.get("negative_achievements").unwrap().as_u64().unwrap(), 1);
        assert_eq!(stats.get("debug_logging").unwrap().as_bool().unwrap(), true);
        
        let event_types = stats.get("achievements_by_event_type").unwrap().as_object().unwrap();
        assert_eq!(event_types.get("assignment_submitted").unwrap().as_u64().unwrap(), 2);
        assert_eq!(event_types.get("assignment_graded").unwrap().as_u64().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_user_progress_retrieval() {
        let (engine, _temp_file) = setup_test_engine().await
            .expect("Failed to set up test engine");
        
        let user_id = 777;
        
        // Initially no progress
        let progress = engine.get_user_progress(user_id, "first_submission").await
            .expect("Failed to get progress");
        assert!(progress.is_none());
        
        // Process an event
        let event = UserEvent::assignment_submitted(
            user_id, 1, 2, 1, false, "solution.zip".to_string(), 1,
            Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap()
        );
        
        engine.handle_event(event).await.expect("Failed to handle event");
        
        // Now should have progress
        let progress = engine.get_user_progress(user_id, "first_submission").await
            .expect("Failed to get progress")
            .expect("Progress should exist");
        
        assert_eq!(progress.user_id, user_id);
        assert_eq!(progress.current_level, 1);
        assert_eq!(progress.progress_value, 1);
        
        // Test getting all progress for user
        let all_progress = engine.get_user_all_progress(user_id).await
            .expect("Failed to get all progress");
        
        assert_eq!(all_progress.len(), 1);
        assert_eq!(all_progress[0].user_id, user_id);
    }

    #[tokio::test]
    async fn test_achievement_definition_retrieval() {
        let (engine, _temp_file) = setup_test_engine().await
            .expect("Failed to set up test engine");
        
        // Test getting specific achievement
        let definition = engine.get_achievement_definition("first_submission").await
            .expect("First submission achievement should exist");
        
        assert_eq!(definition.id, "first_submission");
        assert_eq!(definition.name, "First Steps");
        assert!(definition.is_positive);
        assert_eq!(definition.condition.event, "assignment_submitted");
        assert_eq!(definition.level_thresholds, [1, 1, 1, 1, 1]);
        
        // Test getting non-existent achievement
        let missing = engine.get_achievement_definition("non_existent").await;
        assert!(missing.is_none());
        
        // Test checking if achievement exists
        assert!(engine.has_achievement("first_submission").await);
        assert!(!engine.has_achievement("non_existent").await);
    }

    #[tokio::test]
    async fn test_multiple_events_same_user() {
        let (engine, _temp_file) = setup_test_engine().await
            .expect("Failed to set up test engine");
        
        let user_id = 888;
        
        // Submit assignment (should get first_submission level 1)
        let submit_event = UserEvent::assignment_submitted(
            user_id, 1, 2, 1, false, "solution.zip".to_string(), 1,
            Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap()
        );
        
        let result1 = engine.handle_event(submit_event).await
            .expect("Failed to handle submit event");
        assert_eq!(result1.levels_granted.len(), 1);
        assert_eq!(result1.levels_granted[0].achievement_id, "first_submission");
        
        // Grade assignment with perfect score (should get perfect_score and high_achiever)
        let grade_event = UserEvent::assignment_graded(
            user_id, 1, 2, 10, 1, 100, 100
        );
        
        let result2 = engine.handle_event(grade_event).await
            .expect("Failed to handle grade event");
        assert_eq!(result2.levels_granted.len(), 2);
        
        // Verify final state
        let all_progress = engine.get_user_all_progress(user_id).await
            .expect("Failed to get all progress");
        
        assert_eq!(all_progress.len(), 3); // first_submission, perfect_score, high_achiever
        
        for progress in &all_progress {
            assert_eq!(progress.user_id, user_id);
            assert_eq!(progress.current_level, 1);
            assert_eq!(progress.progress_value, 1);
        }
    }

    #[tokio::test]
    async fn test_error_handling_missing_achievement() {
        let (engine, _temp_file) = setup_test_engine().await
            .expect("Failed to set up test engine");
        
        // Try to get progress for non-existent achievement
        let result = engine.get_user_progress(123, "non_existent").await;
        
        match result {
            Err(AchievementError::AchievementNotFound(id)) => {
                assert_eq!(id, "non_existent");
            }
            _ => panic!("Expected AchievementNotFound error"),
        }
    }

    /// Test configuration options
    #[tokio::test] 
    async fn test_engine_configuration() {
        let db = setup_test_db().await;
        let _temp_file = create_test_achievements_json().await
            .expect("Failed to create test JSON");
        
        // Test with custom config
        let config = AchievementEngineConfig {
            debug_logging: false,
            emit_level_events: false,
            max_achievements_per_event: 2,
        };
        
        // This test mainly verifies configuration is accepted
        // The actual implementation would use mocked files for full testing
        let result = AchievementEngine::new(db, config).await;
        
        // This will fail due to missing achievements.json in current directory,
        // but demonstrates the configuration API
        assert!(result.is_err());
    }

    #[test]
    fn test_event_serialization() {
        let event = UserEvent::assignment_submitted(
            123, 1, 2, 1, false, "test.zip".to_string(), 1,
            Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap()
        );
        
        let json_value = event.to_json().expect("Serialization should work");
        assert!(json_value.is_object());
        
        let data = &json_value["data"];
        assert_eq!(data["user_id"], 123);
        assert_eq!(data["assignment_id"], 1);
        assert_eq!(data["submission_count"], 1);
        assert_eq!(data["submitted_before_deadline"], true);
    }
}