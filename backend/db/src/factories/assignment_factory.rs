use crate::models::assignment::Assignment;

pub fn make_assignment(
    module_id: i64,
    name: &str,
    description: Option<&str>,
    assignment_type: &str, // Should be either "Assignment" or "Practical"
    available_from: &str,
    due_date: &str,
) -> Assignment {
    let now = chrono::Utc::now().naive_utc().to_string();

    Assignment {
        id: 0,
        module_id,
        name: name.to_string(),
        description: description.map(|d| d.to_string()),
        assignment_type: assignment_type.to_string(),
        available_from: available_from.to_string(),
        due_date: due_date.to_string(),
        created_at: now.clone(),
        updated_at: now,
    }
}
