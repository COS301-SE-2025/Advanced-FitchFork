//WELCOME TO CONCURRENCY HELL, HAVE FUN DEBUGGING LMFAAOOOOOO
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use super::message::EventMessage;

#[derive(Clone)]
pub struct EventBus {
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<EventMessage>>>>, 
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn subscribe(&self, channel: &str) -> broadcast::Receiver<EventMessage> {
        let mut map = self.channels.write().unwrap();
        map.entry(channel.to_string())
            .or_insert_with(|| broadcast::channel(100).0)
            .subscribe()
    }

    pub fn publish(&self, channel: &str, event: EventMessage) {
        if let Some(sender) = self.channels.read().unwrap().get(channel) {
            let _ = sender.send(event);
        }
    }
}
