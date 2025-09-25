use axum::{
    body::Body,
    http::{Request, Response},
};
use std::convert::Infallible;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::client::IntoClientRequest,
};
use tower::make::Shared;
use tower::util::BoxCloneService;
use url::Url;

pub async fn spawn_server(
    app: BoxCloneService<Request<Body>, Response<Body>, Infallible>,
) -> std::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let service = Shared::new(app);

    tokio::spawn(async move {
        axum::serve(listener, service).await.unwrap();
    });

    // small settle delay
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    addr
}

/// Connect to the multiplexed WS entrypoint at `/ws`
/// Auth is via HTTP guard, so pass `Authorization: Bearer <token>` header.
/// (No per-topic path; you subscribe after connecting.)
pub async fn connect_ws(
    addr: &str,
    token: Option<&str>,
) -> Result<
    (
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        axum::http::Response<Option<Vec<u8>>>,
    ),
    tokio_tungstenite::tungstenite::Error,
> {
    let url = Url::parse(&format!("ws://{}/ws", addr)).unwrap();

    // Build the upgrade request and inject Authorization if provided
    let mut req = url.to_string().into_client_request().unwrap();

    if let Some(tok) = token {
        use tokio_tungstenite::tungstenite::http::header::{AUTHORIZATION, HeaderValue};
        let hv = HeaderValue::from_str(&format!("Bearer {}", tok)).unwrap();
        req.headers_mut().insert(AUTHORIZATION, hv);
    }

    connect_async(req).await
}
