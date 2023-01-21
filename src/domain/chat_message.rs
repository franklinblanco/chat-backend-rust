use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Used for Both registering delivered and seen time in messages.
/// The reasoning for this is that a chatroom can have many users
/// and the backend needs to be able to tell when each of them
/// has seen this message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeSensitiveAction {
    pub time: DateTime<Utc>,
    pub by: u32,
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChatMessage {
    from: u32,
    to: u32,
    message: ChatMessageContent,
    /// This must always be there. Since its created.
    time_sent: DateTime<Utc>,
    /// This is a Vec because there can be many recipients.
    time_delivered: Vec<TimeSensitiveAction>,
    /// This is a Vec because there can be many recipients.
    time_seen: Vec<TimeSensitiveAction>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChatMessageContent {
    Text(String),
    Image(Vec<u8>),
    Video(Vec<u8>),
    Audio(Vec<u8>),
}

pub trait Sendable {
    /// Creates a new message, automatically sets the time that the message was sent to the current time in UTC.
    fn new(from: u32, to: u32, message: ChatMessageContent) -> Self;
    /// Sets the time that the message was delivered to the current time in UTC.
    fn delivered(&mut self, by: u32);
    /// Sets the time that the message was seen to the current time in UTC.
    fn seen(&mut self, by: u32);
    /// This returns the content of a given message, the backend might need this in the future.
    fn get_content(&self) -> &ChatMessageContent;
}

impl Sendable for ChatMessage {
    fn new(from: u32, to: u32, message: ChatMessageContent) -> Self {
        Self {
            from,
            to,
            message,
            time_sent: Utc::now(),
            time_delivered: Vec::new(),
            time_seen: Vec::new(),
        }
    }

    fn delivered(&mut self, by: u32) {
        self.time_delivered.push(TimeSensitiveAction::new(by));
    }

    fn seen(&mut self, by: u32) {
        self.time_seen.push(TimeSensitiveAction::new(by));
    }

    fn get_content(&self) -> &ChatMessageContent {
        &self.message
    }
}
