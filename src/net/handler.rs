use std::{net::SocketAddr, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{
    domain::{chat_message::BroadcastMessage, state::AppState},
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
        super::recv::ClientMessageIn::Login(_) => {
            send_message(
                sender,
                super::send::ClientMessageOut::Error("Already Logged in!".into()),
            )
            .await?
        }
        super::recv::ClientMessageIn::Logout => todo!(),
        super::recv::ClientMessageIn::SeeMessages(seen_messages) => {
            see_messages(&state, &user_id, seen_messages).await?;
        }
        super::recv::ClientMessageIn::SendMessage(message) => {
            user_send_message(state, user_id, BroadcastMessage::NewMessageRequest(message)).await?;
            send_message(sender, super::send::ClientMessageOut::MessageSent).await?;
        }
        super::recv::ClientMessageIn::JoinGroup() => todo!(),
        super::recv::ClientMessageIn::LeaveGroup() => todo!(),
        super::recv::ClientMessageIn::FetchMessages() => todo!(),
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
    let user_id = state.remove_connected_client(addr)?;
    state.remove_user_from_all_groups(&user_id)?;
    Ok(())
}
