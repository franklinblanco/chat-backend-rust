use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::extract::ws::{Message, WebSocket};
use chrono::Utc;
use dev_communicators::middleware::user_svc::user_service;
use futures::stream::SplitSink;
use tokio::{sync::Mutex, task::{JoinHandle, self}, time::sleep};

use crate::{
    dao::{chat_room_dao, message_dao::{update_message, get_message, self}},
    domain::{state::AppState, chat_message_update::ChatMessageUpdate, chat_message::TimeSensitiveAction},
    net::{
        error::{SocketError, MUTEX_LOCK_ERROR_MESSAGE},
        recv::ClientMessageIn,
        send::ClientMessageOut,
        utils::send_message,
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
                match send_message(
                    sender_cloned_ref.clone(),
                    ClientMessageOut::MessageRecieved(msg.clone()),
                )
                .await
                {
                    Ok(_) => {
                        let time_delivered = Utc::now();
                        if !cloned_state.does_message_have_updates_in_queue(&msg.id) {
                            // Add message delivered update to queue if it's not already in a queue (basically lock the queue from writes)
                            cloned_state.add_message_update_to_queue(&msg.id, ChatMessageUpdate::Delivered(cloned_user_id, time_delivered));
                            // Update data
                            // NOTE: This part of the code is programmed this way to avoid breaking (Since we're in a while loop, breaking would mean the user would stop recieving messages...)
                            let persisted_message_opt = match message_dao::get_message(&cloned_state.db_conn, &msg.id).await {
                                Ok(persisted_message_opt) => persisted_message_opt,
                                Err(error) => {
                                    println!("Something went wrong in the database while performing a get to the message table. Error: {}", error);
                                    None
                                },
                            };
                            if persisted_message_opt.is_some() {
                                // TODO: Move this to a function call since you'll need to use this for seeing messages as well!
                                let mut persisted_message = persisted_message_opt.unwrap();
                                persisted_message.time_delivered.list.push(TimeSensitiveAction::new(cloned_user_id));
                                match message_dao::update_message(&cloned_state.db_conn, &persisted_message).await {
                                    Ok(_) => {},
                                    Err(error) => println!("Something went wrong in the database while performing an update to the message table. Error: {error}"),
                                };
                                match cloned_state.remove_first_message_update_from_queue(&msg.id) {
                                    Some(_) => {},
                                    None => println!("Error while removing the first message from the message update queue. This should never happen. MessageId: {}", msg.id),
                                };
                            }
                        } else {
                            // Message has updates in queue before this one.
                            // Add it to queue
                            // Wait until this update is first in the queue then execute.
                            let message_update  = ChatMessageUpdate::Delivered(cloned_user_id, time_delivered);
                            cloned_state.add_message_update_to_queue(&msg.id, message_update.clone());
                            while !cloned_state.is_update_first_in_queue(&msg.id, &message_update) {
                                // Wait 70ms
                                sleep(Duration::from_millis(50)).await;    
                            }
                            // After it has been confirmed that the update is first in queue, update db
                            let persisted_message_opt = match message_dao::get_message(&cloned_state.db_conn, &msg.id).await {
                                Ok(persisted_message_opt) => persisted_message_opt,
                                Err(error) => {
                                    println!("Something went wrong in the database while performing a get to the message table. Error: {}", error);
                                    None
                                },
                            };
                            if persisted_message_opt.is_some() {
                                let mut persisted_message = persisted_message_opt.unwrap();
                                persisted_message.time_delivered.list.push(TimeSensitiveAction::new(cloned_user_id));
                                match message_dao::update_message(&cloned_state.db_conn, &persisted_message).await {
                                    Ok(_) => {},
                                    Err(error) => println!("Something went wrong in the database while performing an update to the message table. Error: {error}"),
                                };
                                match cloned_state.remove_first_message_update_from_queue(&msg.id) {
                                    Some(_) => {},
                                    None => println!("Error while removing the first message from the message update queue. This should never happen. MessageId: {}", msg.id),
                                };
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

//TODO: Create chat rooms, create them all in the db, all schemas and everything

/*


let sender_cloned_ref = sender.clone();
        let mut sender_task = tokio::spawn(async move {
            while let Ok(msg) = channel_reciever_handle.recv().await {
                println!("Message inside send task;");
                match send_message(sender_cloned_ref.clone(), ClientMessageOut::MessageRecieved(msg.clone())).await {
                    Ok(_) => {
                        println!("Message sent, \nfrom: {} \ncontent: {:?}", msg.from_id, msg.message);
                        //TODO: Register message was sent (in db)?
                    },
                    Err(error) => {
                        println!("{error}");
                        break;
                    },
                };
            }
        });

        let cloned_user_id = user_id.clone();
        let broadcast_sender = state.get_cloned_broadcast_sender_to_chat_room(&chat_room_id)?;
        let reciever_cloned_ref = reciever.clone();
        // This here spawns a new task that will get the messages that our client sends to the broadcast list and send them to
        // all the connected clients registered on the broadcast.
        let mut reciever_task = tokio::spawn(async move {
            while let Some(Ok(message)) = reciever_cloned_ref.lock().await.next().await {
                println!("Message inside recv task");
                match interpret_message(message) {
                    Ok(client_message_in) => {
                        if let ClientMessageIn::SendMessage(chat_message) = client_message_in {
                            let _ = broadcast_sender.send(ChatMessage::new(cloned_user_id, chat_message));
                        }
                        // Else it's not a message to be sent
                    },
                    Err(error) => println!("{error}"),
                }
            }
        });

        println!("done");
        // If any one of the tasks run to completion, we abort the other.
        //tokio::select! {
        //    _ = (&mut sender_task) => reciever_task.abort(),
        //    _ = (&mut reciever_task) => sender_task.abort(),
        //};


*/
