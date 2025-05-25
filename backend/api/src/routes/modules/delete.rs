use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use db::pool;
use crate::auth::AuthUser;
use crate::response::ApiResponse;
use crate::routes::modules::post::AssignLecturersRequest;




/// DELETE /api/modules/:module_id/lecturers
///
/// Remove one or more users from the lecturers of a module. Admin only.
///
/// ### Request Body
/// ```json
/// {
///   "user_ids": [1, 2]
/// }
/// ```
///
/// ### Responses
/// - `200 OK`
/// - `400 Bad Request` (empty list)
/// - `403 Forbidden` (non-admin)
/// - `404 Not Found` (module or user not found)
/// - `409 Conflict` (some users not assigned)
pub async fn remove_lecturers(axum::extract::Path(module_id): axum::extract::Path<i64>, AuthUser(claims): AuthUser, Json(body): Json<AssignLecturersRequest>, )-> impl IntoResponse {
    if !claims.admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "You do not have permission to perform this action".into(),
            }),
        );
    }

    if body.user_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Request must include a non-empty list of user_ids".into(),
            }),
        );
    }

    let pool = pool::get();

    let module_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM modules WHERE id = ?)"
    )
        .bind(module_id)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

    if !module_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Module not found".into(),
            }),
        );
    }

    let mut not_assigned = Vec::new();
    for &user_id in &body.user_ids {
        let user_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = ?)"
        )
            .bind(user_id)
            .fetch_one(pool)
            .await
            .unwrap_or(false);

        if !user_exists {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()> {
                    success: false,
                    data: None,
                    message: format!("User with ID {} does not exist", user_id),
                }),
            );
        }

        let is_assigned: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM module_lecturers WHERE module_id = ? AND user_id = ?)"
        ).bind(module_id).bind(user_id).fetch_one(pool).await.unwrap_or(false);

        if is_assigned {
            let _ = sqlx::query(
                "DELETE FROM module_lecturers WHERE module_id = ? AND user_id = ?"
            ).bind(module_id).bind(user_id).execute(pool).await;
        } else {
            not_assigned.push(user_id);
        }
    }

    if not_assigned.is_empty() {
        (
            StatusCode::OK,
            Json(ApiResponse::<()> {
                success: true,
                data: None,
                message: "Lecturers removed from module successfully".into(),
            })
        )
    } else {
        (
            StatusCode::CONFLICT,
            Json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Some users are not lecturers for this module!".into(),
            })
        )
    }
}
