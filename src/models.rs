use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from: String,
    pub to: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub message_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageType {
    Direct,
    Broadcast,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMessage {
    pub to: String,
    pub content: String,
    #[serde(default)]
    pub message_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMessage {
    pub id: String,
    pub from: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterRegionMessage {
    pub message: Message,
    pub source_region: String,
    pub target_region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthMessage {
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum WsMessage {
    Auth { user_id: String, token: String },
    Message(ClientMessage),
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsResponse {
    AuthSuccess { user_id: String },
    AuthFailure { reason: String },
    Message(ServerMessage),
    Error { message: String },
    Pong,
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::Direct
    }
}

impl Message {
    pub fn new(from: String, to: String, content: String, message_type: MessageType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from,
            to,
            content,
            timestamp: Utc::now(),
            message_type,
        }
    }
}
