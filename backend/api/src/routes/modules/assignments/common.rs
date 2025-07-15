use serde::{Serialize, Deserialize};
use db::models::assignment::Model as AssignmentModel;

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub id: String,
    pub filename: String,
    pub path: String,
    pub file_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssignmentResponse {
    pub id: i64,
    pub module_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub assignment_type: String,
    pub status: String,
    pub available_from: String,
    pub due_date: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<AssignmentModel> for AssignmentResponse {
    fn from(a: AssignmentModel) -> Self {
        Self {
            id: a.id,
            module_id: a.module_id,
            name: a.name,
            description: a.description,
            assignment_type: a.assignment_type.to_string(),
            status: a.status.to_string(),
            available_from: a.available_from.to_rfc3339(),
            due_date: a.due_date.to_rfc3339(),
            created_at: a.created_at.to_rfc3339(),
            updated_at: a.updated_at.to_rfc3339(),
        }
    }
}


#[derive(Debug, Deserialize)]
pub struct AssignmentRequest {
    pub name: String,
    pub description: Option<String>,
    pub assignment_type: String,
    pub status: String,
    pub available_from: String,
    pub due_date: String,
}