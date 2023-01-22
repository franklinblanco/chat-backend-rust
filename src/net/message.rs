use serde::{Deserialize, Serialize};
use serde_json::Value;

/// This is what gets sent across a socket. No matter if it comes from the client or the
/// Server. This is what gets put in Message::Text(HERE).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ClientMessage {
    pub head: String,
    #[serde(skip_serializing_if = "Value::is_null")]
    pub body: Value,
}
