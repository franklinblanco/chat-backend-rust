use std::{collections::HashMap, net::SocketAddr, sync::Mutex};

use chat_types::domain::{chat_message::BroadcastMessage, chat_message_update::ChatMessageUpdate};
use sqlx::MySqlPool;
use tokio::sync::broadcast::{self, Receiver, Sender};

use chat_types::domain::error::{MUTEX_LOCK_ERROR_MESSAGE, SocketError};

use super::chat_room_channel::ChatRoomChannel;

const MAX_CONCURRENT_ROOM_CAPACITY: usize = 150;

#[derive(Debug)]
pub struct AppState {
    /// Rooms holds a map of roomId and the ChatRoomChannel object, pretty much just a list of all the ACTIVE chat rooms.
    /// Active chat rooms mean that there is at least one user connected that belongs to that chat room. If there are
    /// 0 participants of a certain group connected to a socket, the chat room must be deleted from memory.
    pub rooms: Mutex<HashMap<u32, ChatRoomChannel>>, // An id of the room & the room object (that also holds a list of all the users)
    pub connected_clients: Mutex<HashMap<SocketAddr, u32>>,
    pub user_rooms: Mutex<HashMap<u32, Vec<u32>>>, // An id of the user & a list of chat room ids
    pub conn: reqwest::Client,
    pub db_conn: MySqlPool,
    /// Every time a message is delivered or read, it mus first query this hashmap to see if that message is currently being held by another thread.
    /// Then, If it is, add the message
    pub message_update_queue: Mutex<HashMap<u32, Mutex<Vec<ChatMessageUpdate>>>>,
}

impl AppState {
    pub fn new(db_conn: MySqlPool, client: reqwest::Client) -> Self {
        Self {
            rooms: Default::default(),
            connected_clients: Default::default(),
            conn: client,
            user_rooms: Default::default(),
            db_conn,
            message_update_queue: Mutex::new(HashMap::new()),
        }
    }

    pub fn add_connected_client(
        &self,
        addr: SocketAddr,
        user_id: u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // lock mutex
        let mut connected_clients = self
            .connected_clients
            .lock()
            .expect(MUTEX_LOCK_ERROR_MESSAGE);
        match connected_clients.insert(addr, user_id) {
            Some(_) => Err(SocketError::boxed_error("Existing socket connected client replaced by another user id, this should NOT be happening. FATAL!")),
            None => Ok(()),
        }
    }
    pub fn add_user_with_rooms(
        &self,
        user_id: u32,
        rooms: Vec<u32>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut user_rooms = self.user_rooms.lock().expect(MUTEX_LOCK_ERROR_MESSAGE);
        if user_rooms.contains_key(&user_id) {
            return Err(SocketError::boxed_error("Existing user_rooms was attempted to be replaced by another list of rooms, this usually happens when a user logs in from 2 clients at the same time."));
        };
        match user_rooms.insert(user_id, rooms) {
            Some(_) => Err(SocketError::boxed_error("Existing user_rooms replaced by another list of rooms, this should NOT be happening. FATAL!")),
            None => Ok(()),
        }
    }
    pub fn add_chat_room_channel(
        &self,
        room_id: u32,
        user_id: &u32,
    ) -> Result<Receiver<BroadcastMessage>, Box<dyn std::error::Error + Send + Sync>> {
        let mut chat_rooms = self.rooms.lock().expect(MUTEX_LOCK_ERROR_MESSAGE);
        let (tx, rx) = broadcast::channel(MAX_CONCURRENT_ROOM_CAPACITY);
        let chat_room_channel = ChatRoomChannel::new(tx, vec![], room_id);

        match chat_rooms.get_mut (&room_id) {
            Some(existing_chat_room) => {
                existing_chat_room.participants.push(*user_id);
                Ok(rx)
            },
            None => {
                match chat_rooms.insert(room_id, chat_room_channel) {
                    Some(_) => Err(SocketError::boxed_error(
                        "Existing chat_room replaced by another room, this should NOT be happening. FATAL!",
                    )),
                    None => Ok(rx),
                }
            },
        }
    }
    /*pub fn get_cloned_chat_room_channel(&self, room_id: &u32) -> Option<ChatRoomChannel> {
        let chat_rooms = self.rooms.lock().expect(MUTEX_LOCK_ERROR_MESSAGE);
        chat_rooms.get(room_id).cloned()
    }*/
    pub fn subscribe_to_channel(
        &self,
        room_id: &u32,
    ) -> Result<Receiver<BroadcastMessage>, Box<dyn std::error::Error + Send + Sync>> {
        let chat_rooms = self.rooms.lock().expect(MUTEX_LOCK_ERROR_MESSAGE);
        match chat_rooms.get(room_id) {
            Some(chat_room_channel) => Ok(chat_room_channel.recipient_sockets.subscribe()),
            None => Err(SocketError::boxed_error(
                "No chat rooms found with that id. When attempting to subscribe to a channel.",
            )),
        }
    }
    pub fn get_cloned_broadcast_sender_to_chat_room(
        &self,
        room_id: &u32,
    ) -> Result<Sender<BroadcastMessage>, Box<dyn std::error::Error + Send + Sync>> {
        let chat_rooms = self.rooms.lock().expect(MUTEX_LOCK_ERROR_MESSAGE);
        match chat_rooms.get(room_id) {
            Some(chat_room_channel) => Ok(chat_room_channel.recipient_sockets.clone()),
            None => Err(SocketError::boxed_error("No chat rooms found with that id. When attempting to get cloned sender from channel.")),
        }
    }

    pub fn remove_connected_client(
        &self,
        addr: &SocketAddr,
    ) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        let mut connected_clients = self
            .connected_clients
            .lock()
            .expect(MUTEX_LOCK_ERROR_MESSAGE);
        match connected_clients.remove(addr) {
            Some(removed_user_id) => Ok(removed_user_id),
            None => Err(SocketError::boxed_error("No user tied to that Address.")),
        }
    }
    pub fn remove_user_from_all_groups(
        &self,
        user_id: &u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Get all rooms user is in
        // Remove that entry from the user -> rooms map
        // Go into each ChatRoomChannel and remove the user as a participant
        let mut user_rooms = self.user_rooms.lock().expect(MUTEX_LOCK_ERROR_MESSAGE);
        let rooms_user_is_in = match user_rooms.remove(user_id) {
            Some(rooms) => rooms,
            None => return Err(SocketError::boxed_error("No rooms tied to that user_id.")),
        };
        for room_id in rooms_user_is_in {
            let mut chat_room_channels = self.rooms.lock().expect(MUTEX_LOCK_ERROR_MESSAGE);
            let room = match chat_room_channels.get_mut(&room_id) {
                Some(chat_room_channel) => chat_room_channel,
                None => {
                    return Err(SocketError::boxed_error(
                        "No chat_room_channels found with that room id...",
                    ))
                }
            };
            let user_id_pos_to_remove: usize = match room
                .participants
                .iter()
                .position(|user_in_room_id| user_in_room_id == user_id)
            {
                Some(user_id) => user_id,
                None => {
                    return Err(SocketError::boxed_error(
                        "No participants found with that user_id inside the chat room...",
                    ))
                }
            };
            room.participants.remove(user_id_pos_to_remove);
            if room.participants.is_empty() {
                // TODO: Fix this bug, remove the room instead of the 
                chat_room_channels.remove(&room_id);
            }
        }
        Ok(())
    }
    pub fn get_all_user_chat_rooms(&self, user_id: &u32) -> Option<Vec<u32>> {
        let user_rooms = self.user_rooms.lock().expect(MUTEX_LOCK_ERROR_MESSAGE);
        user_rooms.get(user_id).cloned()
    }
    pub fn does_message_have_updates_in_queue(&self, message_id: &u32) -> bool {
        match self
            .message_update_queue
            .lock()
            .expect(MUTEX_LOCK_ERROR_MESSAGE)
            .get(message_id)
        {
            Some(_) => true,
            None => false,
        }
    }
    pub fn add_message_update_to_queue(&self, message_id: &u32, update: ChatMessageUpdate) {
        let mut message_update_queue_lock = self
            .message_update_queue
            .lock()
            .expect(MUTEX_LOCK_ERROR_MESSAGE);
        let message_update_queue = match message_update_queue_lock.get(message_id) {
            Some(message_update_queue) => message_update_queue,
            None => {
                message_update_queue_lock.insert(*message_id, Mutex::new(vec![update]));
                return;
            }
        };
        let mut message_updates = message_update_queue.lock().expect(MUTEX_LOCK_ERROR_MESSAGE);
        message_updates.push(update);
    }
    pub fn remove_first_message_update_from_queue(
        &self,
        message_id: &u32,
    ) -> Option<ChatMessageUpdate> {
        let mut message_update_queue_lock = self
            .message_update_queue
            .lock()
            .expect(MUTEX_LOCK_ERROR_MESSAGE);
        let mut message_update_queue = message_update_queue_lock
            .get_mut(message_id)?
            .lock()
            .expect(MUTEX_LOCK_ERROR_MESSAGE);
        if message_update_queue.len() == 0 {
            drop(message_update_queue);
            message_update_queue_lock.remove(message_id);
            return None;
        }
        let removed_message = message_update_queue.remove(0);
        // delete message update queue if it's empty
        if message_update_queue.len() == 0 {
            drop(message_update_queue);
            message_update_queue_lock.remove(message_id);
        }
        Some(removed_message)
    }
    pub fn is_update_first_in_queue(&self, message_id: &u32, update: &ChatMessageUpdate) -> bool {
        let mut message_update_queue_lock = self
            .message_update_queue
            .lock()
            .expect(MUTEX_LOCK_ERROR_MESSAGE);
        let message_update_queue = match message_update_queue_lock.get(message_id) {
            Some(message_update_queue) => message_update_queue,
            None => return false,
        }
        .lock()
        .expect(MUTEX_LOCK_ERROR_MESSAGE);
        if message_update_queue.len() == 0 {
            drop(message_update_queue);
            message_update_queue_lock.remove(message_id);
            return false;
        }
        message_update_queue.first().unwrap() == update
    }
}
