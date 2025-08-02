use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use validator::Validate;
use sea_orm::{
    IntoActiveModel,
    ActiveModelTrait,
    ColumnTrait,
    Condition,
    DatabaseConnection,
    EntityTrait,
    QueryFilter,
    Set,
};
use db::models::module::{
    self,
    ActiveModel as ModuleActiveModel,
    Column as ModuleCol,
    Entity as ModuleEntity,
};
use crate::response::ApiResponse;
use crate::routes::modules::common::{ModuleRequest, ModuleResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// PUT /api/modules/{module_id}
///
/// Update the details of a specific module by its ID.  
/// Only accessible by admin users.
///
/// ### Request Body
/// ```json
/// {
///   "code": "CS101",
///   "year": 2024,
///   "description": "Introduction to Computer Science",
///   "credits": 15
/// }
/// ```
///
/// ### Validation Rules
/// - `code`: must be in format ABC123 (3 uppercase letters + 3 digits)
/// - `year`: must be current year or later
/// - `description`: must be at most 1000 characters
/// - `credits`: must be a positive number
///
/// ### Responses
///
/// - `200 OK`  
/// ```json
/// {
///   "success": true,
///   "data": {
///     "id": 1,
///     "code": "CS101",
///     "year": 2024,
///     "description": "Introduction to Computer Science",
///     "credits": 15,
///     "created_at": "2024-01-01T00:00:00Z",
///     "updated_at": "2024-01-01T00:00:00Z"
///   },
///   "message": "Module updated successfully"
/// }
/// ```
///
/// - `400 Bad Request`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Module code must be in format ABC123"
/// }
/// ```
///
/// - `403 Forbidden`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "You do not have permission to perform this action"
/// }
/// ```
///
/// - `404 Not Found`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Module not found"
/// }
/// ```
///
/// - `409 Conflict`  
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "Module code already exists"
/// }
/// ```
pub async fn edit_module(
    State(db): State<DatabaseConnection>,
    Path(module_id): Path<i64>,
    Json(req): Json<ModuleRequest>,
) -> impl IntoResponse {
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<ModuleResponse>::error(error_message)),
        );
    }

    let duplicate = ModuleEntity::find()
        .filter(
            Condition::all()
                .add(ModuleCol::Code.eq(req.code.clone()))
                .add(ModuleCol::Id.ne(module_id)),
        )
        .one(&db)
        .await;

    if let Ok(Some(_)) = duplicate {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::<ModuleResponse>::error("Module code already exists")),
        );
    }

    let updated_module = ModuleActiveModel {
        id: Set(module_id),
        code: Set(req.code.clone()),
        year: Set(req.year),
        description: Set(req.description.clone()),
        credits: Set(req.credits),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };

    match updated_module.update(&db).await {
        Ok(module) => (
            StatusCode::OK,
            Json(ApiResponse::success(ModuleResponse::from(module), "Module updated successfully")),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<ModuleResponse>::error("Failed to update module")),
        ),
    }
}


#[derive(Debug, Deserialize, Validate)]
pub struct BulkUpdateRequest {
    #[validate(length(min = 1, message = "At least one module ID is required"))]
    pub module_ids: Vec<i64>,
    
    #[validate(range(min = 2024, message = "Year must be at least 2024"))]
    pub year: Option<i32>,
    
    #[validate(length(max = 1000, message = "Description must be at most 1000 characters"))]
    pub description: Option<String>,
    
    #[validate(range(min = 1, message = "Credits must be positive"))]
    pub credits: Option<i32>,
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

/// PUT /api/modules/bulk
///
/// Bulk update multiple modules by their IDs.
/// Only accessible by admin users.
///
/// ### Request Body
/// ```json
/// {
///   "module_ids": [1, 2, 3],
///   "year": 2025,
///   "description": "Updated description",
///   "credits": 20
/// }
/// ```
///
/// ### Rules
/// - `code` cannot be modified via this route
/// - All fields (`year`, `description`, `credits`) are optional
/// - Empty/null fields are ignored (won't overwrite existing values)
///
/// ### Responses
///
/// - `200 OK`
/// ```json
/// {
///   "success": true,
///   "data": {
///     "updated": 2,
///     "failed": [
///       { "id": 3, "error": "Module not found" }
///     ]
///   },
///   "message": "Updated 2/3 modules"
/// }
/// ```
///
/// - `400 Bad Request`
/// ```json
/// {
///   "success": false,
///   "data": null,
///   "message": "No module IDs provided"
/// }
/// ```
pub async fn bulk_edit_modules(
    State(db): State<DatabaseConnection>,
    Json(raw_value): Json<Value>,
) -> impl IntoResponse {
    // First check for forbidden 'code' field
    if let Some(obj) = raw_value.as_object() {
        if obj.keys().any(|k| k.to_lowercase() == "code") {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<BulkUpdateResult>::error("Bulk update cannot change module code")),
            );
        }
    }

    // Then parse and validate the request
    let req: BulkUpdateRequest = match serde_json::from_value(raw_value) {
        Ok(req) => req,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<BulkUpdateResult>::error(format!("Invalid request body: {}", e))),
            );
        }
    };

    // Validate the request
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<BulkUpdateResult>::error(error_message)),
        );
    }

    let mut updated = 0;
    let mut failed = Vec::new();

    for id in &req.module_ids {
        let res = module::Entity::find()
            .filter(module::Column::Id.eq(*id))
            .one(&db)
            .await;

        match res {
            Ok(Some(model)) => {
                let mut active = model.into_active_model();
                let mut has_changes = false;

                // Only update fields that are provided
                if let Some(year) = req.year {
                    active.year = Set(year);
                    has_changes = true;
                }

                if let Some(ref description) = req.description {
                    active.description = Set(Some(description.clone()));
                    has_changes = true;
                }

                if let Some(credits) = req.credits {
                    active.credits = Set(credits);
                    has_changes = true;
                }

                if has_changes {
                    active.updated_at = Set(Utc::now());

                    if active.update(&db).await.is_ok() {
                        updated += 1;
                    } else {
                        failed.push(FailedUpdate {
                            id: *id,
                            error: "Failed to save updated module".into(),
                        });
                    }
                } else {
                    // No changes to apply, count as successful
                    updated += 1;
                }
            }
            Ok(None) => failed.push(FailedUpdate {
                id: *id,
                error: "Module not found".into(),
            }),
            Err(e) => failed.push(FailedUpdate {
                id: *id,
                error: e.to_string(),
            }),
        }
    }

    let result = BulkUpdateResult { updated, failed };
    let message = format!("Updated {}/{} modules", updated, req.module_ids.len());

    (
        StatusCode::OK,
        Json(ApiResponse::success(result, message)),
    )
}