use std::{net::SocketAddr, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;

use crate::domain::state::AppState;

pub async fn handle_message(
    message: Message,
    sender: &mut SplitSink<WebSocket, Message>,
    state: Arc<AppState>,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Message::Text(name) = message {}
    Ok(())
}
