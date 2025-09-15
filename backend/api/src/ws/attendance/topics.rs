pub fn attendance_session_topic(session_id: i64) -> String {
    format!("ws/attendance/sessions/{session_id}")
}
