#[cfg(test)]
mod tests {
    use axum::{
        body::Body as AxumBody,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use serde_json::Value;
    use crate::test_helpers::make_app;

    #[tokio::test]
    async fn health_check_returns_ok_json() {
        let app = make_app();
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