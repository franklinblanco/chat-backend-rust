use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::{stream::SplitSink, SinkExt};
use tokio::sync::Mutex;

use super::{
    error::SocketError,
    message::ClientMessage,
    recv::{ClientMessageIn, Receivable},
    send::{ClientMessageOut, Sendable},
};

/// Este es el metodo para enviar mensajes a un cliente a traves de un websocket
/// Si le pasas un None en el payload tienes que darle un tipo al metodo, ya que
/// El compilador no permite especificarle un metodo default.
pub async fn send_message(
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    message: ClientMessageOut,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Ok(sender
        .lock()
        .await
        .send(Message::Text(serde_json::to_string(
            &message.into_message()?,
        )?))
        .await?)
}

/// use this function to convert a Message::Text() from a client socket connection
/// into a ClientMessage<Payload>
pub fn interpret_message(
    message: Message,
) -> Result<ClientMessageIn, Box<dyn std::error::Error + Send + Sync>> {
    if let Message::Text(txt) = message {
        // txt should be a {"type": "SOMETHING"} or a {"type": "SOMETHING", "payload": {}}
        let client_message: ClientMessage = serde_json::from_str(txt.as_str())?; //Add error message?
        Ok(ClientMessageIn::from_message(client_message)?)
    } else {
        Err(SocketError::boxed_error(
            "Recieved client Message is not of type Text...",
        ))
    }
}
