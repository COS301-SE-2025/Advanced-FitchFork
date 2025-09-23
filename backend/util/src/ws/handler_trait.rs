use super::runtime::WsContext;
use serde::de::DeserializeOwned;

pub use async_trait::async_trait;

#[async_trait]
pub trait WsHandler: Send + Sync + 'static {
    /// The incoming message type your handler understands (tagged enum recommended)
    type In: DeserializeOwned + Send;

    /// Called once after socket is fully set up (presence already registered).
    async fn on_open(&self, _ctx: &WsContext) {}

    /// Called for every parsed text message of type `Self::In`.
    async fn on_message(&self, ctx: &WsContext, msg: Self::In);

    /// Called when the connection is closing (presence will be unregistered *after* this).
    async fn on_close(&self, _ctx: &WsContext) {}
}
