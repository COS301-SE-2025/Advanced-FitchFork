use crate::models::assignment::Assignment;
use crate::models::assignment::AssignmentType;

pub fn make_assignment(
    module_id: i64,
    name: &str,
    description: Option<&str>,
    assignment_type: &str, // Should be either "Assignment" or "Practical" (Defaults to Practical)
    available_from: &str,
    due_date: &str,
) -> Assignment {
    let now = chrono::Utc::now().naive_utc().to_string();

    let assignment_type_enum = match assignment_type.to_lowercase().as_str() {
        "Assignment" => AssignmentType::Assignment,
        "Practical" => AssignmentType::Practical,
        "A" => AssignmentType::Assignment,
        "P" => AssignmentType::Practical,
        _ => AssignmentType::Practical, //If its wrong, default to Practical
    };

    Assignment {
        id: 0,
        module_id,
        name: name.to_string(),
        description: description.map(|d| d.to_string()),
        assignment_type: assignment_type_enum,
        available_from: available_from.to_string(),
        due_date: due_date.to_string(),
        created_at: now.clone(),
        updated_at: now,
    }
}
