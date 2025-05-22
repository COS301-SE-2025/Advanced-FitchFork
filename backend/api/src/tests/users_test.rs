use api::api::users;
use axum::Router;
use rtest::{TestServer, assert_status_ok, test};

#[test]
async fn test_get_users_returns_200() {
    // Setup the app using just the users route
    let app = users::routes();

    // Wrap the app in a test server
    let server = TestServer::new(app);

    // Send a GET request to /users
    let response = server.get("/users").await;

    // Assert that the response is 200 OK
    assert_status_ok!(response);

    // Optionally, print or validate the body
    let body = response.text().await;
    println!("Response body: {}", body);
}
