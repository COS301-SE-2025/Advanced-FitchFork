#[cfg(test)]
mod tests {
    use db::test_utils::setup_test_db;
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use serde_json::Value;
    use crate::test_helpers::make_app;

    #[tokio::test]
    async fn health_check_returns_ok_json() {
        let db = setup_test_db().await; 

        let app = make_app(db);
        let req = Request::builder()
            .method("GET")
            .uri("/api/health")
            .body(AxumBody::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"], "OK");
        assert_eq!(json["message"], "Health check passed");
    }
}