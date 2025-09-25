use serde::Serialize;

/// An event knows its stable name and the topic it belongs to.
pub trait Event: Serialize {
    const NAME: &'static str;
    /// Return the canonical topic path (e.g., "system" or "system:admin").
    fn topic_path(&self) -> String;
}
