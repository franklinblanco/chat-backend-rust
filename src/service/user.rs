use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::extract::ws::{Message, WebSocket};
use chat_types::domain::{
    chat_message::{BroadcastMessage, TimeSensitiveAction},
    chat_message_update::ChatMessageUpdate,
};
use chrono::Utc;
use dev_communicators::middleware::user_svc::user_service;
use futures::stream::SplitSink;
use tokio::{sync::Mutex, task::JoinHandle, time::sleep};

use crate::{
    dao::{chat_room_dao, message_dao},
    domain::state::AppState,
    net::{
        error::{SocketError, MUTEX_LOCK_ERROR_MESSAGE},
        recv::ClientMessageIn,
        send::ClientMessageOut,
        utils::send_message,
    },
    service::message::user_send_message,
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
    state: Arc<AppState>,
    addr: &SocketAddr,
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    message: &ClientMessageIn,
    all_send_tasks: &mut Vec<JoinHandle<()>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        user_service::authenticate_user_with_token(&state.conn, user_for_auth).await?;
    let _ = send_message(sender.clone(), ClientMessageOut::LoggedIn).await;
    let user_id = persisted_user.id.try_into()?;
    // Store user id along with socket
    state.add_connected_client(*addr, user_id)?;
    // Find rooms user belongs to
    let all_user_chat_rooms =
        chat_room_dao::fetch_all_user_chat_rooms(&state.db_conn, user_id).await?;
    println!("{:?}", all_user_chat_rooms);
    let all_user_chat_room_ids: Vec<u32> = all_user_chat_rooms
        .into_iter()
        .map(|room| room.id)
        .collect();

    state.add_user_with_rooms(user_id, all_user_chat_room_ids.clone())?;
    for chat_room_id in all_user_chat_room_ids {
        let _ = state.add_chat_room_channel(chat_room_id, &user_id)?;
        let mut channel_reciever_handle = state.subscribe_to_channel(&chat_room_id)?;

        let sender_cloned_ref = sender.clone();
        let cloned_state = state.clone();
        let cloned_user_id = user_id.clone(); // The recipient's user id

        // This here spawns a new task that will forward messages that get sent to the channel to the client connected to the current socket.
        let sender_task = tokio::spawn(async move {
            while let Ok(msg) = channel_reciever_handle.recv().await {
                let message_to_send_to_client = match msg.clone() {
                    BroadcastMessage::NewMessage(message) => {
                        ClientMessageOut::MessageRecieved(message)
                    }
                    BroadcastMessage::DeliveredUpdate(delivered_update) => {
                        ClientMessageOut::MessageDelivered(delivered_update)
                    }
                    BroadcastMessage::SeenUpdate(seen_update) => {
                        ClientMessageOut::MessageSeen(seen_update)
                    }
                    BroadcastMessage::NewMessageRequest(message_req) => {
                        println!("New message request being sent to individual users. This is prohibited. Aborting client sender thread. Message attempting to be sent: {:?}", message_req);
                        break;
                    }
                };

                match send_message(sender_cloned_ref.clone(), message_to_send_to_client).await {
                    Ok(_) => {
                        // If broadcast message is a new message then persist the message delivered time to the database,
                        // And send it back to the chat room that x user got his message delivered.
                        if let BroadcastMessage::NewMessage(message) = msg {
                            let time_delivered = Utc::now();
                            if !cloned_state.does_message_have_updates_in_queue(&message.id) {
                                // Add message delivered update to queue if it's not already in a queue (basically lock the queue from writes)
                                cloned_state.add_message_update_to_queue(
                                    &message.id,
                                    ChatMessageUpdate::Delivered(cloned_user_id, time_delivered),
                                );
                                // Update data
                                // NOTE: This part of the code is programmed this way to avoid breaking (Since we're in a while loop, breaking would mean the user would stop recieving messages...)
                                let persisted_message_opt = match message_dao::get_message(
                                    &cloned_state.db_conn,
                                    &message.id,
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
                                    persisted_message
                                        .time_delivered
                                        .list
                                        .push(TimeSensitiveAction::new(cloned_user_id));
                                    match message_dao::update_message(&cloned_state.db_conn, &persisted_message).await {
                                        Ok(_) => {
                                            // Broadcast the delivered message to all connected sockets, 
                                            // The idea is that the clients get the same chatmessage,
                                            // Since they already have that MessageId stored, they can handle it as an update
                                            match user_send_message(cloned_state.clone(), cloned_user_id, BroadcastMessage::DeliveredUpdate(persisted_message)).await {
                                                Ok(_) => {},
                                                Err(error) => println!("Error sending a chat message update to a client: {error}"),
                                            };
                                        },
                                        Err(error) => println!("Something went wrong in the database while performing an update to the message table. Error: {error}"),
                                    };
                                    match cloned_state.remove_first_message_update_from_queue(&message.id) {
                                        Some(_) => {},
                                        None => println!("Error while removing the first message from the message update queue. This should never happen. MessageId: {}", message.id),
                                    };
                                }
                            } else {
                                // Message has updates in queue before this one.
                                // Add it to queue
                                // Wait until this update is first in the queue then execute.
                                let message_update =
                                    ChatMessageUpdate::Delivered(cloned_user_id, time_delivered);
                                cloned_state.add_message_update_to_queue(
                                    &message.id,
                                    message_update.clone(),
                                );
                                while !cloned_state
                                    .is_update_first_in_queue(&message.id, &message_update)
                                {
                                    // Wait 70ms
                                    sleep(Duration::from_millis(50)).await;
                                }
                                // After it has been confirmed that the update is first in queue, update db
                                let persisted_message_opt = match message_dao::get_message(
                                    &cloned_state.db_conn,
                                    &message.id,
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
                                    persisted_message
                                        .time_delivered
                                        .list
                                        .push(TimeSensitiveAction::new(cloned_user_id));
                                    match message_dao::update_message(&cloned_state.db_conn, &persisted_message).await {
                                        Ok(_) => {
                                            // Broadcast the delivered message to all connected sockets, 
                                            // The idea is that the clients get the same chatmessage,
                                            // Since they already have that MessageId stored, they can handle it as an update
                                            match user_send_message(cloned_state.clone(), cloned_user_id, BroadcastMessage::DeliveredUpdate(persisted_message)).await {
                                                Ok(_) => {},
                                                Err(error) => println!("Error sending a chat message update to a client: {error}"),
                                            };
                                        },
                                        Err(error) => println!("Something went wrong in the database while performing an update to the message table. Error: {error}"),
                                    };
                                    match cloned_state.remove_first_message_update_from_queue(&message.id) {
                                        Some(_) => {},
                                        None => println!("Error while removing the first message from the message update queue. This should never happen. MessageId: {}", message.id),
                                    };
                                }
                            }
                        }
                    }
                    Err(error) => {
                        println!("{error}");
                        break;
                    }
                };
            }
        });
        all_send_tasks.push(sender_task);
    }
    Ok(())
}
