use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

/// Dummy authentication middleware used for testing and demonstration.
///
/// This middleware is executed before protected route handlers. It simulates authentication
/// by printing a message to the console and optionally allowing request modification
/// (e.g., injecting a mock user into request extensions).
///
/// ### Behavior
/// - Logs that authentication middleware has been triggered.
/// - Calls the next middleware or handler unconditionally (no actual checks).
///
/// ### Usage
/// Apply using `axum::middleware::from_fn(dummy_auth)` on any route that should simulate protection.
pub async fn dummy_auth(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    println!("Dummy auth middleware triggered");

    // Simulate user injection if needed:
    // req.extensions_mut().insert(MockUser { id: 1, admin: true });

    Ok(next.run(req).await)
}

