use std::net::SocketAddr;

use axum::extract::ws::{Message, WebSocket};
use dev_communicators::middleware::user_svc::user_service;
use futures::stream::SplitSink;

use crate::{
    domain::state::AppState,
    net::{
        error::{SocketError, MUTEX_LOCK_ERROR_MESSAGE},
        recv::ClientMessageIn,
    },
};

pub fn is_addr_registered(state: &AppState, addr: &SocketAddr) -> Option<u32> {
    let connected_clients = state
        .connected_clients
        .lock()
        .expect(MUTEX_LOCK_ERROR_MESSAGE);
    connected_clients.get(addr).copied()
}

/// This method performs all necessary network requests to register a socket address with a user id and find the rooms it belongs to.
pub async fn register_addr(
    state: &AppState,
    addr: &SocketAddr,
    sender: &mut SplitSink<WebSocket, Message>,
    message: &ClientMessageIn,
) -> Result<(), Box<dyn std::error::Error>> {
    let user_for_auth = match message {
        ClientMessageIn::Login(user_for_auth) => user_for_auth,
        _ => {
            return Err(SocketError::boxed_error(format!(
                "Non authorized user attempting to perform authed action. Message: {:?}",
                message
            )))
        }
    };

    // Auth user
    let persisted_user =
        user_service::authenticate_user_with_token(&state.conn, &user_for_auth).await?;
    // Store user id along with socket
    state.add_connected_client(addr.clone(), persisted_user.id.try_into().unwrap());
    // Find rooms user belongs to
    //state.add_user_with_rooms(user_id, rooms);

    Ok(())
}

//TODO: Create chat rooms, create them all in the db, all schemas and everything
