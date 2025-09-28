/// Event system for the achievement engine
/// 
/// This module defines all possible user events that can trigger achievement checks.
/// Events are emitted by business logic throughout the application and consumed
/// by the achievement engine to update user progress.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// All possible user events that can trigger achievement checks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum UserEvent {
    /// User submitted an assignment
    AssignmentSubmitted {
        user_id: i64,
        assignment_id: i64,
        module_id: i64,
        attempt: i64,
        submitted_at: DateTime<Utc>,
        is_practice: bool,
        submission_count: u32,
        submitted_before_deadline: bool,
        hours_since_release: f64,
        filename: String,
    },
    
    /// Assignment was graded/marked
    AssignmentGraded {
        user_id: i64,
        assignment_id: i64,
        module_id: i64,
        submission_id: i64,
        attempt: i64,
        score: i32,
        total_marks: i32,
        percentage: f64,
        graded_at: DateTime<Utc>,
    },
    
    /// Code quality analysis completed
    CodeQualityAnalyzed {
        user_id: i64,
        assignment_id: i64,
        module_id: i64,
        submission_id: i64,
        quality_score: f64,
        analyzed_at: DateTime<Utc>,
    },
    
    /// User attended a class/session
    AttendanceRecorded {
        user_id: i64,
        module_id: i64,
        session_id: i64,
        attended: bool,
        excused: bool,
        recorded_at: DateTime<Utc>,
    },
    
    /// Forum/discussion answer was voted on
    ForumAnswerVoted {
        user_id: i64,
        answer_id: i64,
        module_id: i64,
        vote_type: String, // "helpful", "unhelpful"
        voted_at: DateTime<Utc>,
    },
    
    /// Plagiarism was detected in submission
    PlagiarismDetected {
        user_id: i64,
        assignment_id: i64,
        module_id: i64,
        submission_id: i64,
        similarity_percentage: f64,
        detected_at: DateTime<Utc>,
    },
    
    /// Assignment deadline passed without submission
    AssignmentDeadlinePassed {
        user_id: i64,
        assignment_id: i64,
        module_id: i64,
        deadline: DateTime<Utc>,
        submitted: bool,
    },
    
    /// User posted a discussion/forum message
    DiscussionPosted {
        user_id: i64,
        module_id: i64,
        discussion_id: i64,
        posted_at: DateTime<Utc>,
    },
    
    /// User completed a module/course
    ModuleCompleted {
        user_id: i64,
        module_id: i64,
        completion_percentage: f64,
        completed_at: DateTime<Utc>,
    },
}

impl UserEvent {
    /// Get the user ID for this event
    pub fn user_id(&self) -> i64 {
        match self {
            UserEvent::AssignmentSubmitted { user_id, .. } => *user_id,
            UserEvent::AssignmentGraded { user_id, .. } => *user_id,
            UserEvent::CodeQualityAnalyzed { user_id, .. } => *user_id,
            UserEvent::AttendanceRecorded { user_id, .. } => *user_id,
            UserEvent::ForumAnswerVoted { user_id, .. } => *user_id,
            UserEvent::PlagiarismDetected { user_id, .. } => *user_id,
            UserEvent::AssignmentDeadlinePassed { user_id, .. } => *user_id,
            UserEvent::DiscussionPosted { user_id, .. } => *user_id,
            UserEvent::ModuleCompleted { user_id, .. } => *user_id,
        }
    }
    
    /// Get the event type string for condition matching
    pub fn event_type(&self) -> &'static str {
        match self {
            UserEvent::AssignmentSubmitted { .. } => "assignment_submitted",
            UserEvent::AssignmentGraded { .. } => "assignment_graded",
            UserEvent::CodeQualityAnalyzed { .. } => "code_quality_analyzed",
            UserEvent::AttendanceRecorded { .. } => "attendance_recorded",
            UserEvent::ForumAnswerVoted { .. } => "forum_answer_voted",
            UserEvent::PlagiarismDetected { .. } => "plagiarism_detected",
            UserEvent::AssignmentDeadlinePassed { .. } => "assignment_deadline_passed",
            UserEvent::DiscussionPosted { .. } => "discussion_posted",
            UserEvent::ModuleCompleted { .. } => "module_completed",
        }
    }
    
    /// Convert event to JSON Value for condition checking
    pub fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }
    
    /// Get the module ID for this event (if applicable)
    pub fn module_id(&self) -> Option<i64> {
        match self {
            UserEvent::AssignmentSubmitted { module_id, .. } => Some(*module_id),
            UserEvent::AssignmentGraded { module_id, .. } => Some(*module_id),
            UserEvent::CodeQualityAnalyzed { module_id, .. } => Some(*module_id),
            UserEvent::AttendanceRecorded { module_id, .. } => Some(*module_id),
            UserEvent::ForumAnswerVoted { module_id, .. } => Some(*module_id),
            UserEvent::PlagiarismDetected { module_id, .. } => Some(*module_id),
            UserEvent::AssignmentDeadlinePassed { module_id, .. } => Some(*module_id),
            UserEvent::DiscussionPosted { module_id, .. } => Some(*module_id),
            UserEvent::ModuleCompleted { module_id, .. } => Some(*module_id),
        }
    }
}

/// Event builder helpers for common event creation patterns
impl UserEvent {
    /// Create an assignment submission event
    pub fn assignment_submitted(
        user_id: i64,
        assignment_id: i64,
        module_id: i64,
        attempt: i64,
        is_practice: bool,
        filename: String,
        submission_count: u32,
        due_date: DateTime<Utc>,
        available_from: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        let submitted_before_deadline = now <= due_date;
        let duration = now.signed_duration_since(available_from);
        let hours_since_release = duration.num_minutes() as f64 / 60.0;

        UserEvent::AssignmentSubmitted {
            user_id,
            assignment_id,
            module_id,
            attempt,
            submitted_at: now,
            is_practice,
            submission_count,
            submitted_before_deadline,
            hours_since_release,
            filename,
        }
    }
    
    /// Create an assignment graded event
    pub fn assignment_graded(
        user_id: i64,
        assignment_id: i64,
        module_id: i64,
        submission_id: i64,
        attempt: i64,
        score: i32,
        total_marks: i32,
    ) -> Self {
        let percentage = if total_marks > 0 {
            (score as f64 / total_marks as f64) * 100.0
        } else {
            0.0
        };
        
        UserEvent::AssignmentGraded {
            user_id,
            assignment_id,
            module_id,
            submission_id,
            attempt,
            score,
            total_marks,
            percentage,
            graded_at: Utc::now(),
        }
    }
    
    /// Create an attendance recorded event
    pub fn attendance_recorded(
        user_id: i64,
        module_id: i64,
        session_id: i64,
        attended: bool,
        excused: bool,
    ) -> Self {
        UserEvent::AttendanceRecorded {
            user_id,
            module_id,
            session_id,
            attended,
            excused,
            recorded_at: Utc::now(),
        }
    }
    
    /// Create a plagiarism detected event
    pub fn plagiarism_detected(
        user_id: i64,
        assignment_id: i64,
        module_id: i64,
        submission_id: i64,
        similarity_percentage: f64,
    ) -> Self {
        UserEvent::PlagiarismDetected {
            user_id,
            assignment_id,
            module_id,
            submission_id,
            similarity_percentage,
            detected_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    
    #[test]
    fn test_event_type_extraction() {
        let event = UserEvent::assignment_submitted(
            1, 2, 3, 1, false, "test.zip".to_string(), 1,
            Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap()
        );
        
        assert_eq!(event.event_type(), "assignment_submitted");
        assert_eq!(event.user_id(), 1);
        assert_eq!(event.module_id(), Some(3));
    }
    
    #[test]
    fn test_assignment_graded_percentage_calculation() {
        let event = UserEvent::assignment_graded(1, 2, 3, 4, 1, 85, 100);
        
        match event {
            UserEvent::AssignmentGraded { percentage, .. } => {
                assert_eq!(percentage, 85.0);
            }
            _ => panic!("Wrong event type"),
        }
    }
    
    #[test]
    fn test_json_serialization() {
        let event = UserEvent::AttendanceRecorded {
            user_id: 1,
            module_id: 2,
            session_id: 3,
            attended: true,
            excused: false,
            recorded_at: Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
        };
        
        let json_value = event.to_json().expect("Serialization should succeed");
        assert!(json_value.is_object());
        
        // Check that the event data contains the expected fields
        let data = &json_value["data"];
        assert_eq!(data["user_id"], 1);
        assert_eq!(data["attended"], true);
        assert_eq!(data["excused"], false);
    }
}