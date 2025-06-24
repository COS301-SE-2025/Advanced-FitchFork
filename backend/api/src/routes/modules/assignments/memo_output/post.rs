use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use code_runner::create_memo_outputs_for_all_tasks;

use std::sync::Arc;
use tokio::task::spawn;
use tracing::{error, info};

use crate::{
    response::ApiResponse,
};
 use db::connect;
 
 
 
pub async fn generate_memo_output(
    Path((_module_id, assignment_id)): Path<(i64, i64)>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let db = connect().await;

    spawn(async move {
        match create_memo_outputs_for_all_tasks(&db, assignment_id).await {
            Ok(_) => info!("Memo generation complete for assignment {}", assignment_id),
            Err(e) => error!("Memo generation failed for assignment {}: {}", assignment_id, e),
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(ApiResponse::success(
            (),
            "Memo generation has started",
        )),
    )
}