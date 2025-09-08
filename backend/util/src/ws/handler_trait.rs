use serde::de::DeserializeOwned;
use std::future::Future;
use super::runtime::WsContext;

pub trait WsHandler: Send + Sync + 'static {
    /// The incoming message type your handler understands (tagged enum recommended)
    type In: DeserializeOwned + Send;

    /// Called once after socket is fully set up (presence already registered).
    fn on_open(&self, ctx: &WsContext) -> impl Future<Output = ()> + Send {
        async move {
            let _ = ctx;
        }
    }

    /// Called for every parsed text message of type `Self::In`.
    fn on_message(&self, ctx: &WsContext, msg: Self::In) -> impl Future<Output = ()> + Send;

    /// Called when the connection is closing (presence will be unregistered *after* this).
    fn on_close(&self, ctx: &WsContext) -> impl Future<Output = ()> + Send {
        async move {
            let _ = ctx;
        }
    }
}