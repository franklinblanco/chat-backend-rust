use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ChatRoom {
    pub id: u32,
    pub title: String,
    #[serde(rename = "ownerId")]
    pub owner_id: u32,
    #[serde(rename = "timeCreated")]
    pub time_created: DateTime<Utc>,
    #[serde(rename = "lastUpdated")]
    pub last_updated: DateTime<Utc>,
}
