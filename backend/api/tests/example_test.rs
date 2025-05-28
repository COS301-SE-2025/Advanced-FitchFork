use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::util::ServiceExt; // not axum::ServiceExt

use api::routes::routes;

#[tokio::test]
async fn test_health_check() {
    let app = routes();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_example_index() {
    let app = routes();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/example")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}