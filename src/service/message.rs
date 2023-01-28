use std::sync::Arc;

use crate::{
    dao::message_dao::insert_message,
    domain::{
        chat_message::{ChatMessage, ChatSendable, BroadcastMessage},
        state::AppState,
    },
    net::error::SocketError,
};

/// Gets called when a message is recieved from a socket client, this broadcasts it to all the connected sockets
/// And persists it. 
pub async fn user_send_message(
    state: &Arc<AppState>,
    user_id: u32,
    message: BroadcastMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let chat_rooms_user_belongs_to = match state.get_all_user_chat_rooms(&user_id) {
        Some(chat_rooms) => chat_rooms,
        None => return Err(SocketError::boxed_error("User doesn't have any rooms.")),
    };

    // In reality, the NewMessage will never go in this method as it is what comes out
    let to = match &message {
        BroadcastMessage::NewMessageRequest(new_message_req) => new_message_req.to,
        BroadcastMessage::NewMessage(new_message) => {
            println!("BroadCastMessage::NewMessage Variant passed in to user_send_message method. This shouldn't be happening. SoftError");
            new_message.to_id},
        BroadcastMessage::DeliveredUpdate(delivered_update) => delivered_update.to_id,
        BroadcastMessage::SeenUpdate(seen_update) => seen_update.to_id,
    };

    if !chat_rooms_user_belongs_to.contains(&to) {
        return Err(SocketError::boxed_error(
            "User just tried to send a message to a room he doesn't belong to.",
        ));
    }


    let broadcast_sender = state.get_cloned_broadcast_sender_to_chat_room(&to)?;
    match message.clone() {
        BroadcastMessage::NewMessageRequest(new_message_req) => {
            let mut chat_message_to_send = ChatMessage::new(user_id, new_message_req);
            chat_message_to_send.id = insert_message(&state.db_conn, &chat_message_to_send)
                .await?
                .try_into()
                .unwrap();
            let _ = broadcast_sender.send(BroadcastMessage::NewMessage(chat_message_to_send))?;
            return Ok(())
        },
        _ => {}
    };
    let _ = broadcast_sender.send(message)?;
    Ok(())
}

pub async fn see_messages(state: &Arc<AppState>, user_id: &u32, message_ids: Vec<u32>, ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Find messages in DB
    // Make sure they all correspond to the same chat_room
    
    Ok(())
}