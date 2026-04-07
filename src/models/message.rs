use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub channel_id: i64,
    pub author_id: i64,
    pub author_username: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectMessage {
    pub id: i64,
    pub sender_id: i64,
    pub sender_username: String,
    pub recipient_id: i64,
    pub content: String,
    pub created_at: String,
}

/// WebSocket envelope: `{"type":"...","payload":{...}}`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsEnvelope {
    #[serde(rename = "type")]
    pub event_type: String,
    pub payload: Value,
}

impl WsEnvelope {
    pub fn new(event_type: &str, payload: impl Serialize) -> Self {
        Self {
            event_type: event_type.to_string(),
            payload: serde_json::to_value(payload).unwrap_or(Value::Null),
        }
    }
}
