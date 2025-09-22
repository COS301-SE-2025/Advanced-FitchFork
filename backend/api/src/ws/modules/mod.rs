// api/src/ws/modules/mod.rs
use axum::Router;
use util::state::AppState;

pub mod assignments;

pub fn ws_module_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        // keep any existing module-level WS endpoints here (add .route(...) as needed)
        .nest(
            "/{module_id}/assignments",
            assignments::ws_assignment_routes(app_state.clone()),
        )
        .with_state(app_state)
}
