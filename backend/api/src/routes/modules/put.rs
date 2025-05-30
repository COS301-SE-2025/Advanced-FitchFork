use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use validator::Validate;

use sea_orm::{
    ActiveModelTrait,
    ColumnTrait,
    Condition,
    DatabaseConnection,
    EntityTrait,
    QueryFilter,
    Set,
};

use db::{
    connect,
    models::module::{
        ActiveModel as ModuleActiveModel,
        Column as ModuleCol,
        Entity as ModuleEntity,
        Model as Module,
    },
};

use crate::response::ApiResponse;

#[derive(Debug, Deserialize, Validate)]
pub struct EditModuleRequest {
    #[validate(regex(
        path = "MODULE_CODE_REGEX",
        message = "Module code must be in format ABC123"
    ))]
    pub code: String,

    pub year: i32,

    #[validate(length(max = 1000, message = "Description must be at most 1000 characters"))]
    pub description: String,

    #[validate(range(min = 1, message = "Credits must be a positive number"))]
    pub credits: i32,
}

#[derive(Debug, Serialize)]
struct ModuleResponse {
    id: i64,
    code: String,
    year: i32,
    description: String,
    credits: i32,
    created_at: String,
    updated_at: String,
}

impl From<Module> for ModuleResponse {
    fn from(module: Module) -> Self {
        Self {
            id: module.id,
            code: module.code,
            year: module.year,
            description: module.description.unwrap_or_default(),
            credits: module.credits,
            created_at: module.created_at.to_rfc3339(),
            updated_at: module.updated_at.to_rfc3339(),
        }
    }
}

lazy_static::lazy_static! {
    static ref MODULE_CODE_REGEX: regex::Regex = regex::Regex::new("^[A-Z]{3}\\d{3}$").unwrap();
}

/// Updates the details of a specific module by its ID.
///
/// # Arguments
///
/// Arguments are extracted from:
/// - `Path(module_id)`: The ID of the module to be updated (from the URL path).
/// - `Json(req)`: A JSON payload containing the updated fields:
///   - `code` (string, required): The new module code.
///   - `year` (integer, required): The academic year for the module.
///   - `description` (string, required): The updated module description.
///   - `credits` (integer, required): The number of credits for the module.
///
/// # Returns
///
/// Returns an HTTP response indicating the result:
/// - `200 OK`: If the module was successfully updated. Returns the updated module data.
/// - `400 BAD REQUEST`: If any required field (`code`, `year`, `description`, or `credits`) is missing or invalid.
/// - `404 NOT FOUND`: If no module exists with the given ID.
/// - `409 CONFLICT`: If the new module code already exists (violating a unique constraint).
/// - `500 INTERNAL SERVER ERROR`: If an unexpected database error occurs.
///
/// # Response Format
///
/// All responses use the `ApiResponse<ModuleResponse>` structure for consistency.
pub async fn edit_module(
    Path(module_id): Path<i64>,
    Json(req): Json<EditModuleRequest>,
) -> impl IntoResponse {
    if let Err(validation_errors) = req.validate() {
        let error_message = common::format_validation_errors(&validation_errors);
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<ModuleResponse>::error(error_message)),
        );
    }

    let db: DatabaseConnection = connect().await;

    // Check if the module exists
    let module = ModuleEntity::find_by_id(module_id).one(&db).await;
    if let Ok(None) | Err(_) = module {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<ModuleResponse>::error("Module not found")),
        );
    }

    // Check if the new code is already in use by another module
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
        description: Set(Some(req.description.clone())),
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