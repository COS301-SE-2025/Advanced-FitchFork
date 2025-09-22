use axum::{
    body::Body,
    http::{Request, Response},
};
use std::convert::Infallible;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::client::IntoClientRequest,
};
use tower::make::Shared;
use tower::util::BoxCloneService;
use url::Url;

/// Spawns the Axum app on a random local port
pub async fn spawn_server(
    app: BoxCloneService<Request<Body>, Response<Body>, Infallible>,
) -> std::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let service = Shared::new(app);

    tokio::spawn(async move {
        axum::serve(listener, service).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    addr
}

/// Connects to a WebSocket route at `/{topic_path}?token=...`
pub async fn connect_ws(
    addr: &str,
    topic: &str,
    token: &str,
) -> Result<
    (
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        axum::http::Response<Option<Vec<u8>>>,
    ),
    tokio_tungstenite::tungstenite::Error,
> {
    let url = Url::parse(&format!("ws://{}/ws/{}?token={}", addr, topic, token)).unwrap();

    let req = url.to_string().into_client_request().unwrap();
    connect_async(req).await
}
