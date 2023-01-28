use std::sync::Arc;

use crate::{
    dao::message_dao::insert_message,
    domain::{
        chat_message::{ChatMessage, ChatMessageSender, ChatSendable},
        state::AppState,
    },
    net::error::SocketError,
};

/// Gets called when a message is recieved from a socket client, this broadcasts it to all the connected sockets
/// And persists it.
pub async fn user_send_message(
    state: &Arc<AppState>,
    user_id: u32,
    message: ChatMessageSender,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_rooms_user_belongs_to = match state.get_all_user_chat_rooms(&user_id) {
        Some(chat_rooms) => chat_rooms,
        None => return Err(SocketError::boxed_error("User doesn't have any rooms.")),
    };

    if !chat_rooms_user_belongs_to.contains(&message.to) {
        return Err(SocketError::boxed_error(
            "User just tried to send a message to a room he doesn't belong to.",
        ));
    }

    let broadcast_sender = state.get_cloned_broadcast_sender_to_chat_room(&message.to)?;
    let mut chat_message_to_send = ChatMessage::new(user_id, message);
    chat_message_to_send.id = insert_message(&state.db_conn, &chat_message_to_send)
        .await?
        .try_into()
        .unwrap();
    let _ = broadcast_sender.send(chat_message_to_send.clone())?;
    Ok(())
}