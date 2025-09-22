use api::{auth::guards::validate_known_ids, routes::routes, ws::ws_routes};
use axum::{Router, body::Body, http::Request, middleware::from_fn_with_state, response::Response};
use sea_orm::DatabaseConnection;
use std::convert::Infallible;
use tempfile::TempDir;
use tower::ServiceExt;
use tower::util::BoxCloneService;
use util::test_helpers::setup_test_storage_root;
use util::{state::AppState, ws::WebSocketManager};

pub async fn make_test_app() -> (
    BoxCloneService<Request<Body>, Response, Infallible>,
    AppState,
) {
    let db: DatabaseConnection = db::test_utils::setup_test_db().await;
    let ws = WebSocketManager::new();
    let app_state = AppState::new(db, ws);

    let router = Router::new()
        .nest(
            "/api",
            routes(app_state.clone())
                .layer(from_fn_with_state(app_state.clone(), validate_known_ids)),
        )
        .nest("/ws", ws_routes(app_state.clone()))
        .with_state(app_state.clone());

    let boxed = router.into_service().boxed_clone();
    (boxed, app_state)
}

pub async fn make_test_app_with_storage() -> (
    BoxCloneService<Request<Body>, Response, Infallible>,
    AppState,
    TempDir,
) {
    let tmp = setup_test_storage_root();

    let (app, app_state) = make_test_app().await;
    (app, app_state, tmp)
}
