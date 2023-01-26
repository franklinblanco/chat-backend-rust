use std::{net::SocketAddr, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use dev_communicators::middleware::user_svc::user_service;
use futures::stream::SplitSink;
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{
    dao::{chat_room_dao, message_dao::{update_message, get_message}},
    domain::state::AppState,
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
    state: &AppState,
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
                        println!(
                            "Message sent, from: {} \ncontent: {:?}",
                            msg.from_id, msg.message
                        );
                        //TODO: Register message was sent (in db)?
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

    //TODO: Add dao to method calls
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
