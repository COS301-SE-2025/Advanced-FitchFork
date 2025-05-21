use crate::models::assignment::Assignment;

pub fn make_assignment(module_id: i64, name: &str, due_date: Option<&str>) -> Assignment {
    Assignment {
        id: 0,
        module_id,
        name: name.to_string(),
        due_date: due_date.map(|d| d.to_string()),
    }
}
