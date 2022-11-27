use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// A room with 2 or more members
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Room {
    pub id: String,
    pub title: String,
    pub time_created: DateTime<Utc>,
    pub members: Vec<String>,
    pub messages: Vec<String>,
}