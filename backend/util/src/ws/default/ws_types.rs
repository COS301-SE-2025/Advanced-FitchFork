use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum DefaultIncoming {
    Envelope { text: String },
    // any other JSON is ignored; ping is auto-handled by the framework
    Other(serde_json::Value),
}