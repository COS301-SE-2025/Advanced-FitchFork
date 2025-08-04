//! A thread-safe WebSocket manager for topic-based message broadcasting.
//!
//! This manager uses Tokio's in-memory `broadcast` channels to implement a lightweight
//! publish/subscribe system. Each topic maps to its own `broadcast::Sender<String>`,
//! allowing clients to subscribe to messages and the backend to push updates.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Type alias for topic name.
type Topic = String;

/// Sender for a topic's broadcast channel.
type Sender = broadcast::Sender<String>;

/// Receiver for a topic's broadcast channel.
type Receiver = broadcast::Receiver<String>;

/// Manages broadcast channels per topic to support real-time WebSocket communication.
///
/// This structure is shared across threads and tasks using `Arc<RwLock<_>>`.
/// It lazily creates new topics upon first subscription and automatically removes
/// topics when their subscriber count drops to zero (after broadcasting).
#[derive(Clone)]
pub struct WebSocketManager {
    /// The internal map of topics to broadcast senders, protected by `RwLock`.
    pub inner: Arc<RwLock<HashMap<Topic, Sender>>>,
}

impl WebSocketManager {
    /// Creates a new, empty `WebSocketManager`.
    ///
    /// # Example
    /// ```
    /// use util::ws::manager::WebSocketManager;
    ///
    /// let manager = WebSocketManager::new();
    /// ```
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribes to the given topic.
    ///
    /// If the topic doesn't exist yet, it is created with a broadcast buffer of size 100.
    ///
    /// # Arguments
    /// * `topic` - The name of the topic to subscribe to.
    ///
    /// # Returns
    /// A `broadcast::Receiver<String>` that receives all future messages on this topic.
    pub async fn subscribe(&self, topic: &str) -> Receiver {
        let mut map = self.inner.write().await;
        map.entry(topic.to_string())
            .or_insert_with(|| broadcast::channel(100).0)
            .subscribe()
    }

    /// Broadcasts a message to all subscribers of the given topic.
    ///
    /// If the topic does not exist, this is a no-op.
    /// If the topic has zero subscribers after sending, it is removed.
    ///
    /// # Arguments
    /// * `topic` - The topic to broadcast to.
    /// * `msg` - The message to broadcast.
    pub async fn broadcast<T: Into<String>>(&self, topic: &str, msg: T) {
        let mut map = self.inner.write().await;

        if let Some(sender) = map.get(topic) {
            let _ = sender.send(msg.into());

            if sender.receiver_count() == 0 {
                tracing::info!("Removing topic '{topic}' due to no subscribers.");
                map.remove(topic);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn it_broadcasts_to_all_subscribers() {
        let manager = WebSocketManager::new();
        let topic = "test-topic";

        let mut r1 = manager.subscribe(topic).await;
        let mut r2 = manager.subscribe(topic).await;

        manager.broadcast(topic, "hello world").await;

        let msg1 = timeout(Duration::from_millis(50), r1.recv()).await.unwrap().unwrap();
        let msg2 = timeout(Duration::from_millis(50), r2.recv()).await.unwrap().unwrap();

        assert_eq!(msg1, "hello world");
        assert_eq!(msg2, "hello world");
    }

    #[tokio::test]
    async fn it_creates_topic_lazily() {
        let manager = WebSocketManager::new();
        let topic = "lazy-create";

        assert!(
            manager.inner.read().await.get(topic).is_none(),
            "Topic should not exist before subscription"
        );

        let _ = manager.subscribe(topic).await;

        assert!(
            manager.inner.read().await.get(topic).is_some(),
            "Topic should be created after subscribe"
        );
    }

    #[tokio::test]
    async fn broadcast_to_empty_topic_does_not_panic() {
        let manager = WebSocketManager::new();
        manager.broadcast("no-subscribers", "silent").await;
    }

    #[tokio::test]
    async fn topic_is_removed_after_broadcast_if_no_subscribers() {
        let manager = WebSocketManager::new();
        let topic = "ephemeral-topic";

        {
            let _ = manager.subscribe(topic).await;
        } // drop receiver

        manager.broadcast(topic, "cleanup").await;

        let map = manager.inner.read().await;
        assert!(!map.contains_key(topic), "Topic should have been removed");
    }
}
