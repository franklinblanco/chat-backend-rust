use serde::{Deserialize, Serialize};

use super::chat_message::ChatMessage;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClientMessageOut {
    Acknowledge,
    LoggedIn,

    /// Whenever a user sends a message
    MessageSent,
    /// Whenever a user gets a message
    MessageRecieved(ChatMessage),
    MessageDelivered,
    MessageSeen,
}
