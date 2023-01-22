use std::{net::SocketAddr, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;

use crate::{
    domain::state::AppState,
    service::user::{is_addr_registered, register_addr},
};

use super::utils::interpret_message;

pub async fn handle_message(
    message: Message,
    sender: &mut SplitSink<WebSocket, Message>,
    state: Arc<AppState>,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let client_message_in = interpret_message(message)?;

    let user_id = match is_addr_registered(&state, &addr) {
        Some(user_id) => user_id,
        None => return register_addr(&state, &addr, sender, &client_message_in).await,
    };

    Ok(())
}

/// All the logic that needs to happen whenever a client gets disconnected from the server.
pub async fn disconnect_client(
    state: &Arc<AppState>,
    addr: &SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
