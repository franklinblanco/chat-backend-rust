use crate::domain::chat_message::{ChatMessage, TimeSensitiveAction};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Display;

use super::message::ClientMessage;

pub trait Sendable {
    fn into_message(self) -> Result<ClientMessage, Box<dyn std::error::Error>>;
}

/// Message that can be sent from this server to a connected socket
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClientMessageOut {
    Acknowledge,
    LoggedIn,

    /// Whenever a user sends a message
    MessageSent,
    /// Whenever a user gets a message
    MessageRecieved(ChatMessage),
    /// Whenever a message sent by the user gets delivered along with the MessageId
    MessageDelivered(MessageTimeChangeUpdate),
    /// Whenever a message sent by the user gets seen along with the MessageId
    MessageSeen(MessageTimeChangeUpdate),
}
/// Used as a dto to notify the client that a specific message has been seen or delivered
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct MessageTimeChangeUpdate {
    #[serde(rename = "timeUpdate")]
    pub time_update: TimeSensitiveAction,
    #[serde(rename = "chatMessageId")]
    pub chat_message_id: u128,
}

impl Display for ClientMessageOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientMessageOut::Acknowledge => write!(f, "ACKNOWLEDGE"),
            ClientMessageOut::LoggedIn => write!(f, "LOGGED IN"),
            ClientMessageOut::MessageSent => write!(f, "MESSAGE SENT"),
            ClientMessageOut::MessageRecieved(_) => write!(f, "MESSAGE RECIEVED"),
            ClientMessageOut::MessageDelivered(_) => write!(f, "MESSAGE DELIVERED"),
            ClientMessageOut::MessageSeen(_) => write!(f, "MESSAGE SEEN"),
        }
    }
}

impl Sendable for ClientMessageOut {
    fn into_message(self) -> Result<ClientMessage, Box<(dyn std::error::Error)>> {
        let head = self.to_string();
        match self {
            ClientMessageOut::Acknowledge => Ok(ClientMessage {
                head,
                body: Value::Null,
            }),
            ClientMessageOut::LoggedIn => Ok(ClientMessage {
                head,
                body: Value::Null,
            }),
            ClientMessageOut::MessageSent => Ok(ClientMessage {
                head,
                body: Value::Null,
            }),
            ClientMessageOut::MessageRecieved(chat_message) => Ok(ClientMessage {
                head,
                body: serde_json::to_value(chat_message)?,
            }),
            ClientMessageOut::MessageDelivered(delivered_info) => Ok(ClientMessage {
                head,
                body: serde_json::to_value(delivered_info)?,
            }),
            ClientMessageOut::MessageSeen(seen_info) => Ok(ClientMessage {
                head,
                body: serde_json::to_value(seen_info)?,
            }),
        }
    }
}
