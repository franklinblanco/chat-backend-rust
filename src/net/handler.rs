use std::{net::SocketAddr, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use chat_types::{domain::chat_message::BroadcastMessage, dto::{server_in::ServerMessageIn, server_out::ServerMessageOut}};
use futures::stream::SplitSink;
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{
    domain::state::AppState,
    service::{
        message::{see_messages, user_send_message},
        user::{is_addr_registered, register_addr},
    },
};

use super::utils::{interpret_message, send_message};

pub async fn handle_message(
    message: Message,
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    state: Arc<AppState>,
    addr: SocketAddr,
    all_send_tasks: &mut Vec<JoinHandle<()>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    let client_message_in = match interpret_message(message) {
        Ok(res) => res,
        Err(err) => return Err(err),
    };

    let user_id = match is_addr_registered(&state, &addr) {
        Some(user_id) => user_id,
        None => {
            return register_addr(
                state.clone(),
                &addr,
                sender,
                &client_message_in,
                all_send_tasks,
            )
            .await
        }
    };

    match client_message_in {
        ServerMessageIn::Login(_) => {
            send_message(
                sender,
                ServerMessageOut::Error("Already Logged in!".into()),
            )
            .await?
        }
        ServerMessageIn::Logout => return Ok(()), //TODO: Make this method, should be easy, just disconnect client?
        ServerMessageIn::SeeMessages(seen_messages) => {
            see_messages(&state, &user_id, seen_messages).await?;
        }
        ServerMessageIn::SendMessage(message) => {
            user_send_message(state, user_id, BroadcastMessage::NewMessageRequest(message)).await?;
            send_message(sender, ServerMessageOut::MessageSent).await?;
        }
        ServerMessageIn::FetchMessages() => todo!(),
    };

    Ok(())
}

/// All the logic that needs to happen whenever a client gets disconnected from the server.
pub async fn disconnect_client(
    state: &Arc<AppState>,
    addr: &SocketAddr,
    send_tasks: Vec<JoinHandle<()>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for send_task in send_tasks {
        send_task.abort();
    }
    match state.remove_connected_client(addr) {
        Ok(user_id) => state.remove_user_from_all_groups(&user_id)?,
        Err(error) => return Err(error),
    };
    
    Ok(())
}
