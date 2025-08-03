use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use util::state::AppState;
use validator::Validate;
use sea_orm::{
    ActiveModelTrait,
    ColumnTrait,
    Condition,
    EntityTrait,
    QueryFilter,
    Set,
};
use db::models::module::{
    ActiveModel as ModuleActiveModel,
    Column as ModuleCol,
    Entity as ModuleEntity,
};
use crate::response::ApiResponse;
use crate::routes::modules::common::{ModuleRequest, ModuleResponse};

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
    State(state): State<AppState>,
    Path(module_id): Path<i64>,
    Json(req): Json<ModuleRequest>,
) -> impl IntoResponse {
    let db = state.db();

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
        .one(db)
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

    match updated_module.update(db).await {
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