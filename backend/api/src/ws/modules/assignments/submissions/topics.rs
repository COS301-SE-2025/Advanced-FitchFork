pub fn submission_topic(module_id: i64, assignment_id: i64, user_id: i64) -> String {
    format!("ws/modules/{module_id}/assignments/{assignment_id}/submissions/{user_id}")
}

pub fn submission_staff_topic(module_id: i64, assignment_id: i64) -> String {
    format!("ws/modules/{module_id}/assignments/{assignment_id}/submissions")
}