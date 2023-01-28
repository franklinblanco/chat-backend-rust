use std::fmt::Display;

use dev_dtos::dtos::user::user_dtos::UserForAuthenticationDto;
use serde::{Deserialize, Serialize};

use crate::domain::chat_message::ChatMessageSender;

use super::{error::SocketError, message::ClientMessage};

pub trait Receivable {
    fn from_message(
        message: ClientMessage,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized;
}

/// Message that can be recieved from the client to a connected socket
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClientMessageIn {
    Login(UserForAuthenticationDto),
    Logout,

    /// A list of MessageId's that the client reports to see
    SeeMessages(Vec<u32>),
    SendMessage(ChatMessageSender),

    /// Client can send this to server to fetch old messages.
    /// By Old messages I mean: Messages that have already been delivered/seen
    FetchMessages(),

    JoinGroup(),
    LeaveGroup(),
}

impl Display for ClientMessageIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientMessageIn::Login(_) => write!(f, "LOGIN"),
            ClientMessageIn::Logout => write!(f, "LOGOUT"),
            ClientMessageIn::SeeMessages(_) => write!(f, "SEE MESSAGES"),
            ClientMessageIn::SendMessage(_) => write!(f, "SEND MESSAGE"),
            ClientMessageIn::JoinGroup() => write!(f, "JOIN GROUP"),
            ClientMessageIn::LeaveGroup() => write!(f, "LEAVE GROUP"),
            ClientMessageIn::FetchMessages() => write!(f, "FETCH MESSAGES"),
        }
    }
}

impl Receivable for ClientMessageIn {
    fn from_message(
        message: ClientMessage,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match message.head.as_str() {
            "LOGIN" => Ok(Self::Login(serde_json::from_value(message.body)?)),
            "LOGOUT" => Ok(Self::Logout),
            "SEE MESSAGES" => Ok(Self::SeeMessages(serde_json::from_value(message.body)?)),
            "SEND MESSAGE" => Ok(Self::SendMessage(serde_json::from_value(message.body)?)),
            "JOIN GROUP" => Ok(Self::JoinGroup()),
            "LEAVE GROUP" => Ok(Self::LeaveGroup()),
            "FETCH MESSAGES" => Ok(Self::FetchMessages()),
            _ => Err(SocketError::boxed_error(format!(
                "ClientMessage recieved isn't recognized by the server. ClientMessage: {:#?}",
                message
            ))),
        }
    }
}
