/// Achievement service for integrating the achievement engine with the API layer
/// 
/// This service provides a high-level interface for the achievement engine,
/// handling initialization, event processing, and integration with the application state.

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use tokio::sync::OnceCell;
use tracing::{info, error, warn};

use crate::achievement_engine::{AchievementEngine, AchievementEngineConfig, EventProcessingResult, LevelGrant};
use crate::events::UserEvent;

/// Global achievement service instance
static ACHIEVEMENT_SERVICE: OnceCell<AchievementService> = OnceCell::const_new();

/// Service for managing achievements and processing events
#[derive(Clone)]
pub struct AchievementService {
    engine: Arc<AchievementEngine>,
}

impl AchievementService {
    /// Initialize the global achievement service
    pub async fn initialize(
        db: DatabaseConnection,
        config: Option<AchievementEngineConfig>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let config = config.unwrap_or_default();
        
        info!("Initializing achievement service");
        
        let engine = AchievementEngine::new(db, config).await
            .map_err(|e| format!("Failed to create achievement engine: {}", e))?;
        
        let service = Self {
            engine: Arc::new(engine),
        };
        
        ACHIEVEMENT_SERVICE.set(service)
            .map_err(|_| "Achievement service already initialized")?;
        
        info!("Achievement service initialized successfully");
        Ok(())
    }
    
    /// Get the global achievement service instance
    pub fn global() -> Result<&'static Self, &'static str> {
        ACHIEVEMENT_SERVICE.get().ok_or("Achievement service not initialized")
    }
    
    /// Try to get the global achievement service instance, returning None if not initialized
    pub fn try_global() -> Option<&'static Self> {
        ACHIEVEMENT_SERVICE.get()
    }
    
    /// Process a user event (main entry point for business logic)
    pub async fn process_event(&self, event: UserEvent) -> Result<EventProcessingResult, String> {
        let user_id = event.user_id();
        let event_type = event.event_type();
        
        match self.engine.handle_event(event).await {
            Ok(result) => {
                if !result.levels_granted.is_empty() {
                    info!("Achievement service processed event '{}' for user {}: {} achievements updated, {} levels granted",
                        event_type, user_id, result.achievements_updated.len(), result.levels_granted.len());
                    
                    // Here you could emit notifications, webhooks, etc.
                    self.handle_level_grants(&result.levels_granted).await;
                }
                Ok(result)
            }
            Err(e) => {
                error!("Failed to process event '{}' for user {}: {}", event_type, user_id, e);
                Err(e.to_string())
            }
        }
    }
    
    /// Emit a user event (convenience method)
    pub async fn emit_event(&self, event: UserEvent) {
        if let Err(e) = self.process_event(event).await {
            error!("Failed to emit user event: {}", e);
        }
    }
    
    /// Emit an assignment submitted event
    pub async fn emit_assignment_submitted(
        &self,
        user_id: i64,
        assignment_id: i64,
        module_id: i64,
        attempt: i64,
        is_practice: bool,
        filename: String,
        submission_count: u32,
        due_date: chrono::DateTime<chrono::Utc>,
        available_from: chrono::DateTime<chrono::Utc>,
    ) {
        let event = UserEvent::assignment_submitted(
            user_id, 
            assignment_id, 
            module_id, 
            attempt, 
            is_practice, 
            filename, 
            submission_count,
            due_date, 
            available_from
        );
        self.emit_event(event).await;
    }
    
    /// Emit an assignment graded event
    pub async fn emit_assignment_graded(
        &self,
        user_id: i64,
        assignment_id: i64,
        module_id: i64,
        submission_id: i64,
        attempt: i64,
        score: i32,
        total_marks: i32,
    ) {
        let event = UserEvent::assignment_graded(
            user_id, 
            assignment_id, 
            module_id, 
            submission_id, 
            attempt, 
            score, 
            total_marks
        );
        self.emit_event(event).await;
    }
    
    /// Emit an attendance recorded event
    pub async fn emit_attendance_recorded(
        &self,
        user_id: i64,
        module_id: i64,
        session_id: i64,
        attended: bool,
        excused: bool,
    ) {
        let event = UserEvent::attendance_recorded(user_id, module_id, session_id, attended, excused);
        self.emit_event(event).await;
    }
    
    /// Emit a plagiarism detected event
    pub async fn emit_plagiarism_detected(
        &self,
        user_id: i64,
        assignment_id: i64,
        module_id: i64,
        submission_id: i64,
        similarity_percentage: f64,
    ) {
        let event = UserEvent::plagiarism_detected(
            user_id, 
            assignment_id, 
            module_id, 
            submission_id, 
            similarity_percentage
        );
        self.emit_event(event).await;
    }
    
    /// Reload achievement definitions from the database and JSON
    pub async fn reload_achievements(&self) -> Result<(), String> {
        self.engine.reload_achievements().await
            .map_err(|e| e.to_string())
    }
    
    /// Get achievement engine statistics
    pub async fn get_stats(&self) -> std::collections::HashMap<String, serde_json::Value> {
        self.engine.get_engine_stats().await
    }
    
    /// Get user's progress for a specific achievement
    pub async fn get_user_progress(&self, user_id: i64, achievement_id: &str) -> Result<Option<crate::models::achievement_progress::Model>, String> {
        self.engine.get_user_progress(user_id, achievement_id).await
            .map_err(|e| e.to_string())
    }
    
    /// Get all progress for a user
    pub async fn get_user_all_progress(&self, user_id: i64) -> Result<Vec<crate::models::achievement_progress::Model>, String> {
        self.engine.get_user_all_progress(user_id).await
            .map_err(|e| e.to_string())
    }
    
    /// Handle level grants (can be extended for notifications, etc.)
    async fn handle_level_grants(&self, grants: &[LevelGrant]) {
        for grant in grants {
            info!(
                "🏆 User {} earned {} level {} in '{}' ({})", 
                grant.user_id,
                if grant.is_positive { "achievement" } else { "penalty" },
                grant.new_level,
                grant.achievement_name,
                grant.achievement_id
            );
            
            // Here you could:
            // - Send notifications
            // - Emit websocket events
            // - Create database records for notifications
            // - Send webhooks
            // - Update leaderboards
        }
    }
}

/// Convenience function to emit events to the global service if initialized
pub async fn emit_event_global(event: UserEvent) {
    if let Some(service) = AchievementService::try_global() {
        service.emit_event(event).await;
    } else {
        warn!("Achievement service not initialized, event ignored: {}", event.event_type());
    }
}

/// Convenience function to emit assignment submission
pub async fn emit_assignment_submitted_global(
    user_id: i64,
    assignment_id: i64,
    module_id: i64,
    attempt: i64,
    is_practice: bool,
    filename: String,
    submission_count: u32,
    due_date: chrono::DateTime<chrono::Utc>,
    available_from: chrono::DateTime<chrono::Utc>,
) {
    if let Some(service) = AchievementService::try_global() {
        service.emit_assignment_submitted(
            user_id, assignment_id, module_id, attempt, is_practice, 
            filename, submission_count, due_date, available_from
        ).await;
    }
}

/// Convenience function to emit assignment grading
pub async fn emit_assignment_graded_global(
    user_id: i64,
    assignment_id: i64,
    module_id: i64,
    submission_id: i64,
    attempt: i64,
    score: i32,
    total_marks: i32,
) {
    if let Some(service) = AchievementService::try_global() {
        service.emit_assignment_graded(
            user_id, assignment_id, module_id, submission_id, 
            attempt, score, total_marks
        ).await;
    }
}

/// Convenience function to emit attendance recording
pub async fn emit_attendance_recorded_global(
    user_id: i64,
    module_id: i64,
    session_id: i64,
    attended: bool,
    excused: bool,
) {
    if let Some(service) = AchievementService::try_global() {
        service.emit_attendance_recorded(user_id, module_id, session_id, attended, excused).await;
    }
}

/// Convenience function to emit plagiarism detection
pub async fn emit_plagiarism_detected_global(
    user_id: i64,
    assignment_id: i64,
    module_id: i64,
    submission_id: i64,
    similarity_percentage: f64,
) {
    if let Some(service) = AchievementService::try_global() {
        service.emit_plagiarism_detected(
            user_id, assignment_id, module_id, submission_id, similarity_percentage
        ).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_db;
    use crate::achievement_engine::AchievementEngineConfig;

    #[tokio::test]
    async fn test_achievement_service_creation() {
        let db = setup_test_db().await;
        let config = AchievementEngineConfig {
            debug_logging: true,
            ..Default::default()
        };
        
        // This will fail without achievements.json, but shows the API
        let result = AchievementService::initialize(db, Some(config)).await;
        assert!(result.is_err()); // Expected due to missing achievements.json
    }
    
    #[tokio::test]
    async fn test_global_service_access() {
        // Before initialization
        assert!(AchievementService::global().is_err());
        assert!(AchievementService::try_global().is_none());
    }
}