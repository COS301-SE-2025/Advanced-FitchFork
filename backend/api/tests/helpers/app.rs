use axum::{
    body::Body,
    http::Request,
    middleware::from_fn_with_state,
    response::Response,
    Router,
};
use sea_orm::DatabaseConnection;
use util::{config::AppConfig, state::AppState, ws::WebSocketManager};
use api::{
    auth::guards::validate_known_ids,
    routes::routes,
    ws::ws_routes,
};
use tower::util::BoxCloneService;
use tower::ServiceExt;
use std::convert::Infallible;

pub async fn make_test_app() -> (
    BoxCloneService<Request<Body>, Response, Infallible>,
    AppState,
) {
    let db: DatabaseConnection = db::test_utils::setup_test_db().await;
    let _ = AppConfig::from_env(); // Initialize the config singleton
    let ws = WebSocketManager::new();
    let app_state = AppState::new(db, ws);

    let router = Router::new()
        .nest("/api", routes(app_state.clone()).layer(from_fn_with_state(app_state.clone(), validate_known_ids)))
        .nest("/ws", ws_routes(app_state.clone())) 
        .with_state(app_state.clone());

    let boxed = router.into_service().boxed_clone();
    (boxed, app_state)
}
