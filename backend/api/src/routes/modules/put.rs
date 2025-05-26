use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use db::models::module::Module;
use serde::Serialize;

use crate::response::ApiResponse;
#[derive(Debug, Serialize)]

struct ModuleResponse {
    id: i64,
    code: String,
    year: i32,
    description: String,
    credits: i32,
}

impl From<Module> for ModuleResponse {
    fn from(module: db::models::module::Module) -> Self {
        Self {
            id: module.id,
            code: module.code,
            year: module.year,
            description: module.description.unwrap_or_default(),
            credits: module.credits,
        }
    }
    
}


pub async fn edit_module(
    Path(module_id): Path<i64>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let code = req.get("code").and_then(|v| v.as_str());
    let year = req.get("year").and_then(|v| v.as_i64());
    let description = req.get("description").and_then(|v| v.as_str());
    let credits = req.get("credits").and_then(|v| v.as_i64());

    if code.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<ModuleResponse>::error("Module code is expected")),
        );
    }
    if year.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<ModuleResponse>::error("Module year is expected")),
        );
    }
    if description.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<ModuleResponse>::error("Module description is expected")),
        );
    }
    if credits.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<ModuleResponse>::error("Module credits is expected")),
        );
    }

    let code = code.unwrap();
    let year = year.unwrap() as i32;
    let description = description.unwrap();
    let credits = credits.unwrap() as i32;

    match db::models::module::Module::edit(
        Some(db::pool::get()),
        module_id,
        code,
        year,
        description,
        credits,
    )
    .await
    {
        Ok(module) => {
            let res = ModuleResponse::from(module);
            return (
                StatusCode::OK,
                Json(ApiResponse::success(res, "Module updated successfully")),
            );
        }
        Err(e) => {
            if e.to_string().contains("no rows") {
                return (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::<ModuleResponse>::error("Module not found")),
                );
            }
            if e.to_string().contains("constraint failed") {
                return (
                    StatusCode::CONFLICT,
                    Json(ApiResponse::<ModuleResponse>::error("Module code already exists")),
                );
            }
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<ModuleResponse>::error(&format!(
                    "Failed to update module:",
                ))),
            );
        }
    }
}
