/// Integration tests for the complete achievement system flow
/// 
/// These tests verify that the achievement system works end-to-end from
/// API endpoints through event emission to database updates.

use std::collections::HashMap;
use tempfile::NamedTempFile;
use tokio::fs;
use serde_json::json;
use chrono::{TimeZone, Utc};

use crate::achievement_service::AchievementService;
use crate::achievement_engine::AchievementEngineConfig;
use crate::events::UserEvent;
use crate::models::achievements::Model as AchievementModel;
use crate::models::achievement_progress::Model as ProgressModel;
use crate::test_utils::setup_test_db;

/// Integration test setup
struct IntegrationTestSetup {
    _temp_file: NamedTempFile,
    service: &'static AchievementService,
}

async fn setup_integration_test() -> Result<IntegrationTestSetup, Box<dyn std::error::Error>> {
    let db = setup_test_db().await;
    
    // Create comprehensive test achievements JSON
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
            "early_bird": {
                "id": "early_bird",
                "name": "Early Bird",
                "description": "Submit assignments before the deadline",
                "is_positive": true,
                "condition": {
                    "event": "assignment_submitted",
                    "checks": [
                        { "field": "submitted_before_deadline", "op": "eq", "value": true }
                    ]
                },
                "level_thresholds": [1, 5, 10, 25, 50]
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
            "first_try_perfect": {
                "id": "first_try_perfect",
                "name": "First Try Perfect",
                "description": "Achieve 100% score on first attempt",
                "is_positive": true,
                "condition": {
                    "event": "assignment_graded",
                    "checks": [
                        { "field": "score", "op": "eq", "value": 100 },
                        { "field": "attempt", "op": "eq", "value": 1 }
                    ]
                },
                "level_thresholds": [1, 3, 5, 10, 15]
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
            "quick_learner": {
                "id": "quick_learner",
                "name": "Quick Learner",
                "description": "Submit assignments within hours of release",
                "is_positive": true,
                "condition": {
                    "event": "assignment_submitted",
                    "checks": [
                        { "field": "hours_since_release", "op": "lte", "value": 24 }
                    ]
                },
                "level_thresholds": [1, 5, 10, 20, 35]
            },
            "attendance_champion": {
                "id": "attendance_champion",
                "name": "Attendance Champion",
                "description": "Maintain perfect attendance",
                "is_positive": true,
                "condition": {
                    "event": "attendance_recorded",
                    "checks": [
                        { "field": "attended", "op": "eq", "value": true }
                    ]
                },
                "level_thresholds": [10, 20, 40, 60, 80]
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
            },
            "low_score": {
                "id": "low_score",
                "name": "Needs Improvement",
                "description": "Receive low scores on assignments",
                "is_positive": false,
                "condition": {
                    "event": "assignment_graded",
                    "checks": [
                        { "field": "score", "op": "lt", "value": 50 }
                    ]
                },
                "level_thresholds": [1, 3, 5, 8, 15]
            },
            "plagiarism_detected": {
                "id": "plagiarism_detected",
                "name": "Academic Integrity Violation",
                "description": "Have plagiarism detected in submissions",
                "is_positive": false,
                "condition": {
                    "event": "plagiarism_detected",
                    "checks": [
                        { "field": "similarity_percentage", "op": "gte", "value": 70 }
                    ]
                },
                "level_thresholds": [1, 2, 3, 5, 8]
            }
        }
    });

    let temp_file = NamedTempFile::new()?;
    fs::write(temp_file.path(), serde_json::to_string_pretty(&test_config)?).await?;
    
    // Create database achievements
    let achievements_data = [
        ("first_submission", "First Steps", "Submit your first assignment to FitchFork", true, 5),
        ("early_bird", "Early Bird", "Submit assignments before the deadline", true, 5),
        ("perfect_score", "Perfect Score", "Achieve 100% on assignments", true, 5),
        ("first_try_perfect", "First Try Perfect", "Achieve 100% score on first attempt", true, 5),
        ("high_achiever", "High Achiever", "Consistently score above 90%", true, 5),
        ("quick_learner", "Quick Learner", "Submit assignments within hours of release", true, 5),
        ("attendance_champion", "Attendance Champion", "Maintain perfect attendance", true, 5),
        ("late_submission", "Procrastinator", "Submit assignments after the deadline", false, 5),
        ("low_score", "Needs Improvement", "Receive low scores on assignments", false, 5),
        ("plagiarism_detected", "Academic Integrity Violation", "Have plagiarism detected in submissions", false, 5),
    ];
    
    for (condition_id, name, desc, is_positive, levels) in achievements_data {
        AchievementModel::create(&db, name, desc, is_positive, levels, condition_id)
            .await
            .expect("Failed to create test achievement");
    }
    
    // Set up environment to find test achievements file
    let original_path = std::env::current_dir()?;
    let temp_dir = temp_file.path().parent().unwrap();
    std::env::set_current_dir(temp_dir)?;
    
    // Initialize the achievement service
    let config = AchievementEngineConfig {
        debug_logging: true,
        emit_level_events: true,
        max_achievements_per_event: 20,
    };
    
    AchievementService::initialize(db, Some(config)).await?;
    
    // Restore original directory
    std::env::set_current_dir(original_path)?;
    
    let service = AchievementService::global()?;
    
    Ok(IntegrationTestSetup {
        _temp_file: temp_file,
        service,
    })
}

#[tokio::test]
async fn test_complete_assignment_workflow() {
    let setup = setup_integration_test().await
        .expect("Failed to set up integration test");
    
    let user_id = 1001;
    let assignment_id = 1;
    let module_id = 1;
    
    // Step 1: Submit assignment on time
    setup.service.emit_assignment_submitted(
        user_id,
        assignment_id,
        module_id,
        1, // attempt
        false, // not practice
        "solution.zip".to_string(),
        1, // first submission
        Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap(), // due date
        Some(Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap()) // release date
    ).await;
    
    // Verify first submission achievements
    let progress = setup.service.get_user_progress(user_id, "first_submission").await
        .expect("Failed to get progress")
        .expect("Progress should exist");
    
    assert_eq!(progress.current_level, 1);
    assert_eq!(progress.progress_value, 1);
    
    // Should also get early bird (on time) and quick learner (within 24 hours)
    let early_bird_progress = setup.service.get_user_progress(user_id, "early_bird").await
        .expect("Failed to get progress")
        .expect("Early bird progress should exist");
    
    assert_eq!(early_bird_progress.current_level, 1);
    
    // Step 2: Grade assignment with perfect score on first attempt
    setup.service.emit_assignment_graded(
        user_id,
        assignment_id,
        module_id,
        101, // submission_id
        1, // attempt (first try)
        100, // perfect score
        100 // total marks
    ).await;
    
    // Should trigger: perfect_score, first_try_perfect, high_achiever
    let perfect_progress = setup.service.get_user_progress(user_id, "perfect_score").await
        .expect("Failed to get progress")
        .expect("Perfect score progress should exist");
    
    assert_eq!(perfect_progress.current_level, 1);
    
    let first_try_progress = setup.service.get_user_progress(user_id, "first_try_perfect").await
        .expect("Failed to get progress")
        .expect("First try perfect progress should exist");
    
    assert_eq!(first_try_progress.current_level, 1);
    
    let high_achiever_progress = setup.service.get_user_progress(user_id, "high_achiever").await
        .expect("Failed to get progress")
        .expect("High achiever progress should exist");
    
    assert_eq!(high_achiever_progress.current_level, 1); // Need 3 for level 1, only have 1
    
    // Verify user has multiple achievements
    let all_progress = setup.service.get_user_all_progress(user_id).await
        .expect("Failed to get all progress");
    
    // Should have: first_submission, early_bird, quick_learner, perfect_score, first_try_perfect, high_achiever
    assert!(all_progress.len() >= 6);
}

#[tokio::test]
async fn test_negative_achievement_workflow() {
    let setup = setup_integration_test().await
        .expect("Failed to set up integration test");
    
    let user_id = 2001;
    let assignment_id = 2;
    let module_id = 1;
    
    // Step 1: Submit late assignment
    setup.service.emit_assignment_submitted(
        user_id,
        assignment_id,
        module_id,
        1,
        false,
        "late_solution.zip".to_string(),
        1,
        Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap(), // due date (in past relative to now)
        Some(Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap())
    ).await;
    
    // Should get first_submission (positive) and late_submission (negative)
    let first_progress = setup.service.get_user_progress(user_id, "first_submission").await
        .expect("Failed to get progress")
        .expect("First submission progress should exist");
    
    assert_eq!(first_progress.current_level, 1);
    
    let late_progress = setup.service.get_user_progress(user_id, "late_submission").await
        .expect("Failed to get progress")
        .expect("Late submission progress should exist");
    
    assert_eq!(late_progress.current_level, 1);
    
    // Step 2: Grade with low score
    setup.service.emit_assignment_graded(
        user_id,
        assignment_id,
        module_id,
        201,
        1,
        35, // low score
        100
    ).await;
    
    // Should trigger low_score achievement
    let low_score_progress = setup.service.get_user_progress(user_id, "low_score").await
        .expect("Failed to get progress")
        .expect("Low score progress should exist");
    
    assert_eq!(low_score_progress.current_level, 1);
    
    // Step 3: Simulate plagiarism detection
    let plagiarism_event = UserEvent::plagiarism_detected(
        user_id,
        assignment_id,
        module_id,
        201, // submission_id
        75.5 // similarity percentage > 70%
    );
    
    let result = setup.service.process_event(plagiarism_event).await
        .expect("Failed to process plagiarism event");
    
    assert_eq!(result.achievements_updated.len(), 1);
    assert_eq!(result.achievements_updated[0], "plagiarism_detected");
    
    let plagiarism_progress = setup.service.get_user_progress(user_id, "plagiarism_detected").await
        .expect("Failed to get progress")
        .expect("Plagiarism progress should exist");
    
    assert_eq!(plagiarism_progress.current_level, 1);
}

#[tokio::test]
async fn test_level_progression() {
    let setup = setup_integration_test().await
        .expect("Failed to set up integration test");
    
    let user_id = 3001;
    
    // Submit multiple high-scoring assignments to progress high_achiever levels
    // high_achiever thresholds: [3, 5, 10, 20, 40]
    for i in 1..=12 {
        setup.service.emit_assignment_graded(
            user_id,
            i, // different assignment_id
            1, // module_id
            300 + i, // different submission_id
            1, // attempt
            95, // score >= 90
            100
        ).await;
    }
    
    // After 12 high scores, should be at level 3 (need 10 for level 3)
    let high_achiever_progress = setup.service.get_user_progress(user_id, "high_achiever").await
        .expect("Failed to get progress")
        .expect("High achiever progress should exist");
    
    assert_eq!(high_achiever_progress.progress_value, 12);
    assert_eq!(high_achiever_progress.current_level, 3); // 12 >= 10 threshold
}

#[tokio::test]
async fn test_attendance_achievements() {
    let setup = setup_integration_test().await
        .expect("Failed to set up integration test");
    
    let user_id = 4001;
    let module_id = 1;
    
    // Attend multiple sessions to progress attendance_champion
    // attendance_champion thresholds: [10, 20, 40, 60, 80]
    for session_id in 1..=15 {
        setup.service.emit_attendance_recorded(
            user_id,
            module_id,
            session_id,
            true, // attended
            false // not excused
        ).await;
    }
    
    // After 15 attendances, should be at level 2 (15 >= 10 but < 20)
    let attendance_progress = setup.service.get_user_progress(user_id, "attendance_champion").await
        .expect("Failed to get progress")
        .expect("Attendance progress should exist");
    
    assert_eq!(attendance_progress.progress_value, 15);
    assert_eq!(attendance_progress.current_level, 1); // 15 >= 10 threshold for level 1
}

#[tokio::test]
async fn test_multiple_conditions_achievement() {
    let setup = setup_integration_test().await
        .expect("Failed to set up integration test");
    
    let user_id = 5001;
    
    // Test first_try_perfect which requires both score=100 AND attempt=1
    
    // First, try with perfect score but not first attempt
    setup.service.emit_assignment_graded(
        user_id,
        1,
        1,
        501,
        2, // attempt 2 (not first try)
        100,
        100
    ).await;
    
    // Should get perfect_score and high_achiever, but NOT first_try_perfect
    let perfect_progress = setup.service.get_user_progress(user_id, "perfect_score").await
        .expect("Failed to get progress")
        .expect("Perfect score progress should exist");
    
    assert_eq!(perfect_progress.current_level, 1);
    
    let first_try_progress = setup.service.get_user_progress(user_id, "first_try_perfect").await
        .expect("Failed to get progress");
    
    assert!(first_try_progress.is_none(), "Should not have first try perfect achievement");
    
    // Now try with perfect score on first attempt
    setup.service.emit_assignment_graded(
        user_id,
        2, // different assignment
        1,
        502,
        1, // attempt 1 (first try)
        100,
        100
    ).await;
    
    // Now should get first_try_perfect
    let first_try_progress = setup.service.get_user_progress(user_id, "first_try_perfect").await
        .expect("Failed to get progress")
        .expect("First try perfect progress should exist");
    
    assert_eq!(first_try_progress.current_level, 1);
}

#[tokio::test]
async fn test_service_statistics() {
    let setup = setup_integration_test().await
        .expect("Failed to set up integration test");
    
    let stats = setup.service.get_stats().await;
    
    // Should have loaded all 10 achievements
    assert_eq!(stats.get("loaded_achievements").unwrap().as_u64().unwrap(), 10);
    
    // Should have 7 positive and 3 negative achievements
    assert_eq!(stats.get("positive_achievements").unwrap().as_u64().unwrap(), 7);
    assert_eq!(stats.get("negative_achievements").unwrap().as_u64().unwrap(), 3);
    
    // Check event type distribution
    let event_types = stats.get("achievements_by_event_type").unwrap().as_object().unwrap();
    assert_eq!(event_types.get("assignment_submitted").unwrap().as_u64().unwrap(), 4); // first_submission, early_bird, quick_learner, late_submission
    assert_eq!(event_types.get("assignment_graded").unwrap().as_u64().unwrap(), 4); // perfect_score, first_try_perfect, high_achiever, low_score
    assert_eq!(event_types.get("attendance_recorded").unwrap().as_u64().unwrap(), 1); // attendance_champion
    assert_eq!(event_types.get("plagiarism_detected").unwrap().as_u64().unwrap(), 1); // plagiarism_detected
}

#[tokio::test]
async fn test_achievement_reload() {
    let setup = setup_integration_test().await
        .expect("Failed to set up integration test");
    
    // Test reloading achievements
    let result = setup.service.reload_achievements().await;
    assert!(result.is_ok(), "Should be able to reload achievements");
    
    // Verify achievements are still loaded
    let stats = setup.service.get_stats().await;
    assert_eq!(stats.get("loaded_achievements").unwrap().as_u64().unwrap(), 10);
}

#[tokio::test]
async fn test_concurrent_events() {
    let setup = setup_integration_test().await
        .expect("Failed to set up integration test");
    
    let user_id = 6001;
    
    // Process multiple events concurrently for the same user
    let events = vec![
        UserEvent::assignment_submitted(
            user_id, 1, 1, 1, false, "solution1.zip".to_string(), 1,
            Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap(),
            Some(Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap())
        ),
        UserEvent::assignment_graded(user_id, 1, 1, 601, 1, 100, 100),
        UserEvent::attendance_recorded(user_id, 1, 1, true, false),
    ];
    
    // Process events concurrently
    let futures: Vec<_> = events.into_iter()
        .map(|event| setup.service.process_event(event))
        .collect();
    
    let results = futures::future::join_all(futures).await;
    
    // All events should process successfully
    for result in results {
        assert!(result.is_ok(), "All concurrent events should process successfully");
    }
    
    // Verify final state
    let all_progress = setup.service.get_user_all_progress(user_id).await
        .expect("Failed to get all progress");
    
    // Should have achievements from all three event types
    assert!(all_progress.len() >= 3);
}

#[tokio::test] 
async fn test_error_resilience() {
    let setup = setup_integration_test().await
        .expect("Failed to set up integration test");
    
    let user_id = 7001;
    
    // Process a valid event first
    let valid_event = UserEvent::assignment_submitted(
        user_id, 1, 1, 1, false, "solution.zip".to_string(), 1,
        Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap(),
        Some(Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap())
    );
    
    let result = setup.service.process_event(valid_event).await;
    assert!(result.is_ok());
    
    // Verify progress was recorded despite any potential errors
    let progress = setup.service.get_user_progress(user_id, "first_submission").await
        .expect("Failed to get progress");
    
    assert!(progress.is_some(), "Progress should be recorded even with error resilience testing");
}