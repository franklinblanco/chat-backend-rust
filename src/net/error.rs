use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SocketError {
    pub message: String,
}
impl SocketError {
    pub fn boxed_error(message: impl ToString) -> Box<Self> {
        Box::new(Self {
            message: message.to_string(),
        })
    }
}
impl Display for SocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for SocketError {}

pub const MUTEX_LOCK_ERROR_MESSAGE: &str = "Fatal Error, mutex was attempted to be aqcuired while another thread had it and panicked while it was locked.";
