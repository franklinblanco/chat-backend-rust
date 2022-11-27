use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use super::{user::User, room::Room};


/// A message to be sent in chat
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub id: String,
    pub text: String,
    pub sender: User,
    pub recipient: Room,
    pub time_sent: DateTime<Utc>,
    pub time_recieved: DateTime<Utc>,
}