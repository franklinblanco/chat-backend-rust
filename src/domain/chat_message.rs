use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{
    mysql::{MySqlTypeInfo, MySqlValueRef},
    FromRow, MySql,
};

/// Used for Both registering delivered and seen time in messages.
/// The reasoning for this is that a chatroom can have many users
/// and the backend needs to be able to tell when each of them
/// has seen this message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeSensitiveAction {
    pub time: DateTime<Utc>,
    pub by: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeSensitiveActionVec {
    pub list: Vec<TimeSensitiveAction>,
}
impl sqlx::Type<MySql> for TimeSensitiveActionVec {
    fn type_info() -> MySqlTypeInfo {
        <str as sqlx::Type<MySql>>::type_info()
    }
}
impl sqlx::Encode<'_, MySql> for TimeSensitiveActionVec {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> sqlx::encode::IsNull {
        let json_str = serde_json::to_string(self).unwrap();
        <&str as sqlx::Encode<MySql>>::encode(&json_str, buf)
    }
}
impl sqlx::Decode<'_, MySql> for TimeSensitiveActionVec {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        match <&str as sqlx::Decode<MySql>>::decode(value).map(ToOwned::to_owned) {
            Ok(json_str) => match serde_json::from_str(json_str.as_str()) {
                Ok(time_sensitive_action) => Ok(time_sensitive_action),
                Err(error) => Err(Box::new(error)),
            },
            Err(error) => Err(error),
        }
    }
}

impl TimeSensitiveAction {
    pub fn new(by: u32) -> Self {
        Self {
            time: Utc::now(),
            by,
        }
    }
}

/// Base message for chat rooms.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, FromRow)]
pub struct ChatMessage {
    pub id: u32,
    /// User id
    #[serde(rename = "fromId")]
    pub from_id: u32,
    /// ChatRoom id (Not a user id)
    #[serde(rename = "toId")]
    pub to_id: u32,
    pub message: ChatMessageContent,
    /// This must always be there. Since its created.
    #[serde(rename = "timeSent")]
    pub time_sent: DateTime<Utc>,
    /// This is a Vec because there can be many recipients.
    #[serde(rename = "timeDelivered")]
    pub time_delivered: TimeSensitiveActionVec,
    /// This is a Vec because there can be many recipients.
    #[serde(rename = "timeSeen")]
    pub time_seen: TimeSensitiveActionVec,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChatMessageContent {
    Text(String),
    Image(Vec<u8>),
    Video(Vec<u8>),
    Audio(Vec<u8>),
}

impl sqlx::Type<MySql> for ChatMessageContent {
    fn type_info() -> MySqlTypeInfo {
        <str as sqlx::Type<MySql>>::type_info()
    }
}
impl sqlx::Encode<'_, MySql> for ChatMessageContent {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> sqlx::encode::IsNull {
        let json_str = serde_json::to_string(self).unwrap();
        <&str as sqlx::Encode<MySql>>::encode(&json_str, buf)
    }
}
impl sqlx::Decode<'_, MySql> for ChatMessageContent {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        match <&str as sqlx::Decode<MySql>>::decode(value).map(ToOwned::to_owned) {
            Ok(json_str) => match serde_json::from_str(json_str.as_str()) {
                Ok(time_sensitive_action) => Ok(time_sensitive_action),
                Err(error) => Err(Box::new(error)),
            },
            Err(error) => Err(error),
        }
    }
}

pub trait ChatSendable {
    /// Creates a new message, automatically sets the time that the message was sent to the current time in UTC.
    fn new(from: u32, message: ChatMessageSender) -> Self;
    /// Sets the time that the message was delivered to the current time in UTC.
    fn delivered(&mut self, by: u32);
    /// Sets the time that the message was seen to the current time in UTC.
    fn seen(&mut self, by: u32);
    /// This returns the content of a given message, the backend might need this in the future.
    fn get_content(&self) -> &ChatMessageContent;
}

impl ChatSendable for ChatMessage {
    fn new(from_id: u32, message: ChatMessageSender) -> Self {
        Self {
            id: 0, //TODO: Assign a random number
            from_id,
            to_id: message.to,
            message: message.message,
            time_sent: Utc::now(),
            time_delivered: TimeSensitiveActionVec { list: Vec::new() },
            time_seen: TimeSensitiveActionVec { list: Vec::new() },
        }
    }

    fn delivered(&mut self, by: u32) {
        self.time_delivered.list.push(TimeSensitiveAction::new(by));
    }

    fn seen(&mut self, by: u32) {
        self.time_seen.list.push(TimeSensitiveAction::new(by));
    }

    fn get_content(&self) -> &ChatMessageContent {
        &self.message
    }
}

/// This is what clients use to send messages (DTO)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChatMessageSender {
    pub message: ChatMessageContent,
    pub to: u32,
}

/// This is what should be sent across the broadcast channels
/// All of them use the same object so that the client can just replace its own
/// Copy of it with the server's authority.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BroadcastMessage {
    NewMessageRequest(ChatMessageSender),
    NewMessage(ChatMessage),
    DeliveredUpdate(ChatMessage),
    SeenUpdate(ChatMessage),
}
