//! A thread-safe WebSocket manager for topic-based message broadcasting, with presence tracking.
//!
//! Uses Tokio broadcast channels per topic. Also tracks user presence per topic
//! to enable duplicate-notification suppression on the server side.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

/// Type alias for topic name.
type Topic = String;

/// Sender for a topic's broadcast channel.
type Sender = broadcast::Sender<String>;

/// Receiver for a topic's broadcast channel.
type Receiver = broadcast::Receiver<String>;

/// Manages broadcast channels per topic to support real-time WebSocket communication.
///
/// - Lazily creates broadcast channels per topic on first subscription
/// - Removes topics when their subscriber count drops to zero after sending
/// - Tracks user presence per topic using a refcount (supports multiple tabs)
#[derive(Clone, Default)]
pub struct WebSocketManager {
    /// Map of topics to broadcast senders.
    pub inner: Arc<RwLock<HashMap<Topic, Sender>>>,
    /// Presence map: topic -> (user_id -> refcount)
    presence: Arc<RwLock<HashMap<Topic, HashMap<i64, usize>>>>,
}

impl WebSocketManager {
    /// Creates a new, empty `WebSocketManager`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribes to the given topic, creating it if necessary.
    pub async fn subscribe(&self, topic: &str) -> Receiver {
        let mut map = self.inner.write().await;
        map.entry(topic.to_string())
            .or_insert_with(|| broadcast::channel(100).0)
            .subscribe()
    }

    /// Broadcasts a message to all subscribers of `topic`.
    ///
    /// If the topic does not exist, it's a no-op.
    /// If the topic has zero subscribers after sending, it is removed.
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

    // -------------------- Presence API --------------------

    /// Increment presence refcount for `user_id` on `topic`.
    /// Call this when a socket subscribes/opens for that topic.
    pub async fn register(&self, topic: &str, user_id: i64) {
        let mut p = self.presence.write().await;
        let entry = p.entry(topic.to_string()).or_default();
        *entry.entry(user_id).or_insert(0) += 1;
    }

    /// Decrement presence refcount for `user_id` on `topic`.
    /// Call this when a socket closes for that topic.
    pub async fn unregister(&self, topic: &str, user_id: i64) {
        let mut p = self.presence.write().await;
        if let Some(users) = p.get_mut(topic) {
            if let Some(cnt) = users.get_mut(&user_id) {
                if *cnt > 1 {
                    *cnt -= 1;
                } else {
                    users.remove(&user_id);
                }
            }
            if users.is_empty() {
                p.remove(topic);
            }
        }
    }

    /// Returns `true` if `user_id` currently has at least one active subscription to `topic`.
    pub async fn is_user_present_on(&self, topic: &str, user_id: i64) -> bool {
        let p = self.presence.read().await;
        p.get(topic).and_then(|m| m.get(&user_id)).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{Duration, timeout};

    #[tokio::test]
    async fn it_broadcasts_to_all_subscribers() {
        let manager = WebSocketManager::new();
        let topic = "test-topic";

        let mut r1 = manager.subscribe(topic).await;
        let mut r2 = manager.subscribe(topic).await;

        manager.broadcast(topic, "hello world").await;

        let msg1 = timeout(Duration::from_millis(50), r1.recv())
            .await
            .unwrap()
            .unwrap();
        let msg2 = timeout(Duration::from_millis(50), r2.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(msg1, "hello world");
        assert_eq!(msg2, "hello world");
    }

    #[tokio::test]
    async fn it_creates_topic_lazily() {
        let manager = WebSocketManager::new();
        let topic = "lazy-create";
        assert!(manager.inner.read().await.get(topic).is_none());
        let _ = manager.subscribe(topic).await;
        assert!(manager.inner.read().await.get(topic).is_some());
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
        assert!(!map.contains_key(topic));
    }

    #[tokio::test]
    async fn presence_register_unregister_and_query() {
        let m = WebSocketManager::new();
        let topic = "p";
        assert!(!m.is_user_present_on(topic, 7).await);
        m.register(topic, 7).await;
        assert!(m.is_user_present_on(topic, 7).await);
        m.register(topic, 7).await; // refcount 2
        m.unregister(topic, 7).await; // refcount 1
        assert!(m.is_user_present_on(topic, 7).await);
        m.unregister(topic, 7).await; // refcount 0
        assert!(!m.is_user_present_on(topic, 7).await);
    }
}
