use std::{sync::Arc, time::Duration};

use chat_types::domain::{
    chat_message::{BroadcastMessage, ChatMessage, ChatSendable, TimeSensitiveAction},
    chat_message_update::ChatMessageUpdate,
};
use chrono::Utc;
use tokio::time::sleep;

use crate::{
    dao::message_dao::{self, insert_message},
    domain::state::AppState,
    net::error::SocketError,
};

/// Gets called when a message is recieved from a socket client, this broadcasts it to all the connected sockets
/// And persists it.
pub async fn user_send_message(
    state: Arc<AppState>,
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
            new_message.to_id
        }
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
            return Ok(());
        }
        _ => {}
    };
    let _ = broadcast_sender.send(message)?;
    Ok(())
}

/// Method called when the client sends to the server that they saw the message(s) sent to them
/// TODO: Avoid repeat seen
pub async fn see_messages(
    state: &Arc<AppState>,
    user_id: &u32,
    message_ids: Vec<u32>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if message_ids.len() == 0 {
        return Err(SocketError::boxed_error(
            "Empty list of message_ids to see... What are you trying to do?",
        ));
    };
    // Find messages in DB
    let persisted_messages =
        message_dao::fetch_messages_with_ids(&state.db_conn, &message_ids).await?;
    // Make sure they all correspond to the same chat_room
    if persisted_messages.len() == 0 || persisted_messages.len() != message_ids.len() {
        return Err(SocketError::boxed_error(
            "MessageIds don't exist in the databse.",
        ));
    };
    // Grab the room id of the first message
    let room_id = persisted_messages.get(0).unwrap().to_id;
    // Check that all of them are to the same room, if they aren't throw error
    if !persisted_messages
        .iter()
        .all(|persisted_message| persisted_message.to_id == room_id)
    {
        return Err(SocketError::boxed_error(
            "All Messages don't have the same roomId",
        ));
    };
    // Check that the user belongs to this chat room
    let chat_rooms_user_belongs_to = match state.get_all_user_chat_rooms(&user_id) {
        Some(chat_rooms) => chat_rooms,
        None => return Err(SocketError::boxed_error("User doesn't have any rooms.")),
    };
    if !chat_rooms_user_belongs_to.contains(&room_id) {
        return Err(SocketError::boxed_error(
            "User just tried to see a message in a room he doesn't belong to.",
        ));
    };

    // Persist the seen messages to the database
    // Spawn a new task for this
    let cloned_state = state.clone();
    let cloned_user_id = user_id.clone();
    let cloned_message_ids = message_ids.clone();
    tokio::task::spawn(async move {
        let time_seen = Utc::now();
        for message_id in cloned_message_ids {
            if !cloned_state.does_message_have_updates_in_queue(&message_id) {
                // Add message delivered update to queue if it's not already in a queue (basically lock the queue from writes)
                cloned_state.add_message_update_to_queue(
                    &message_id,
                    ChatMessageUpdate::Seen(cloned_user_id, time_seen),
                );
                // Update data
                // NOTE: This part of the code is programmed this way to avoid breaking (Since we're in a while loop, breaking would mean the user would stop recieving messages...)
                let persisted_message_opt = match message_dao::get_message(
                    &cloned_state.db_conn,
                    &message_id,
                )
                .await
                {
                    Ok(persisted_message_opt) => persisted_message_opt,
                    Err(error) => {
                        println!("Something went wrong in the database while performing a get to the message table. Error: {}", error);
                        None
                    }
                };
                if persisted_message_opt.is_some() {
                    let mut persisted_message = persisted_message_opt.unwrap();
                    if persisted_message
                        .time_seen
                        .list
                        .iter()
                        .any(|time_seen| time_seen.by == cloned_user_id)
                    {
                        println!("User is attempting to read a message that is already read by that user. Breaking the loop...");
                        break;
                    }
                    persisted_message.time_seen.list.push(TimeSensitiveAction {
                        time: time_seen,
                        by: cloned_user_id,
                    });
                    match message_dao::update_message(&cloned_state.db_conn, &persisted_message).await {
                            Ok(_) => {
                                // Broadcast the delivered message to all connected sockets, 
                                // The idea is that the clients get the same chatmessage,
                                // Since they already have that MessageId stored, they can handle it as an update
                                match user_send_message(cloned_state.clone(), cloned_user_id, BroadcastMessage::SeenUpdate(persisted_message)).await {
                                    Ok(_) => {},
                                    Err(error) => println!("Error sending a chat message update to a client: {error}"),
                                };
                            },
                            Err(error) => println!("Something went wrong in the database while performing an update to the message table. Error: {error}"),
                        };
                    match cloned_state.remove_first_message_update_from_queue(&message_id) {
                            Some(_) => {},
                            None => println!("Error while removing the first message from the message update queue. This should never happen. MessageId: {}", message_id),
                        };
                }
            } else {
                // Message has updates in queue before this one.
                // Add it to queue
                // Wait until this update is first in the queue then execute.
                let message_update = ChatMessageUpdate::Seen(cloned_user_id, time_seen);
                cloned_state.add_message_update_to_queue(&message_id, message_update.clone());
                while !cloned_state.is_update_first_in_queue(&message_id, &message_update) {
                    // Wait 50ms
                    sleep(Duration::from_millis(50)).await;
                }
                // After it has been confirmed that the update is first in queue, update db
                let persisted_message_opt = match message_dao::get_message(
                    &cloned_state.db_conn,
                    &message_id,
                )
                .await
                {
                    Ok(persisted_message_opt) => persisted_message_opt,
                    Err(error) => {
                        println!("Something went wrong in the database while performing a get to the message table. Error: {}", error);
                        None
                    }
                };
                if persisted_message_opt.is_some() {
                    let mut persisted_message = persisted_message_opt.unwrap();
                    if persisted_message
                        .time_seen
                        .list
                        .iter()
                        .any(|time_seen| time_seen.by == cloned_user_id)
                    {
                        println!("User is attempting to read a message that is already read by that user. Breaking the loop...");
                        break;
                    }
                    persisted_message
                        .time_seen
                        .list
                        .push(TimeSensitiveAction::new(cloned_user_id));
                    match message_dao::update_message(&cloned_state.db_conn, &persisted_message).await {
                            Ok(_) => {
                                // Broadcast the seen message to all connected sockets, 
                                // The idea is that the clients get the same chatmessage,
                                // Since they already have that MessageId stored, they can handle it as an update
                                match user_send_message(cloned_state.clone(), cloned_user_id, BroadcastMessage::SeenUpdate(persisted_message)).await {
                                    Ok(_) => {},
                                    Err(error) => println!("Error sending a chat message update to a client: {error}"),
                                };
                            },
                            Err(error) => println!("Something went wrong in the database while performing an update to the message table. Error: {error}"),
                        };
                    match cloned_state.remove_first_message_update_from_queue(&message_id) {
                            Some(_) => {},
                            None => println!("Error while removing the first message from the message update queue. This should never happen. MessageId: {}", message_id),
                        };
                }
            }
        }
        // Pasted
    });

    Ok(())
}
