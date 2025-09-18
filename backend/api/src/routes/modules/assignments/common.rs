//! Assignment request and response models.
//!
//! Provides data structures for creating, reading, and managing assignments, including bulk operations.
//!
//! - `AssignmentRequest` – payload for creating/updating a single assignment.
//! - `AssignmentResponse` – returned assignment details.
//! - `File` – metadata for assignment-associated files.
//! - `BulkDeleteRequest` – payload for deleting multiple assignments.
//! - `BulkUpdateRequest` – payload for updating multiple assignments.
//! - `BulkUpdateResult` and `FailedUpdate` – results of bulk update operations.

use db::models::{
    assignment::Model as AssignmentModel,
    assignment_file, // 👈 for conversion to File
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub id: String,
    pub filename: String,
    pub path: String,
    pub file_type: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<assignment_file::Model> for File {
    fn from(f: assignment_file::Model) -> Self {
        File {
            id: f.id.to_string(),
            filename: f.filename,
            path: f.path,
            file_type: f.file_type.to_string(),
            created_at: f.created_at.to_rfc3339(),
            updated_at: f.updated_at.to_rfc3339(),
        }
    }
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
    pub available_from: String,
    pub due_date: String,
}

#[derive(Deserialize)]
pub struct BulkDeleteRequest {
    pub assignment_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct BulkUpdateRequest {
    pub assignment_ids: Vec<i64>,

    // Optional fields to apply
    pub status: Option<String>,
    pub available_from: Option<String>,
    pub due_date: Option<String>,
}

#[derive(Serialize)]
pub struct BulkUpdateResult {
    pub updated: usize,
    pub failed: Vec<FailedUpdate>,
}

#[derive(Serialize)]
pub struct FailedUpdate {
    pub id: i64,
    pub error: String,
}
