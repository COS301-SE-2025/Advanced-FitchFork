use serde::{Serialize, Deserialize};
use db::models::assignment::Model as AssignmentModel;

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub id: String,
    pub filename: String,
    pub path: String,
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
    pub available_from: String,
    pub due_date: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<AssignmentModel> for AssignmentResponse {
    fn from(model: AssignmentModel) -> Self {
        Self {
            id: model.id,
            module_id: model.module_id,
            name: model.name,
            description: model.description,
            assignment_type: model.assignment_type.to_string(),
            available_from: model.available_from.to_rfc3339(),
            due_date: model.due_date.to_rfc3339(),
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AssignmentRequest {
    pub name: String,
    pub description: Option<String>,
    pub assignment_type: String,
    pub available_from: String,
    pub due_date: String,
}