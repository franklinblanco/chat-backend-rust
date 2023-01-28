use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChatMessageUpdate {
    /// User id that got the message delivered to, and the time it was delivered.
    Delivered(u32, DateTime<Utc>),
    /// User id that saw the message, and the time it was seen.
    Seen(u32, DateTime<Utc>),
}
