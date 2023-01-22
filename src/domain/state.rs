use std::{collections::HashMap, net::SocketAddr, sync::Mutex};

use sqlx::MySqlPool;

use crate::{net::error::{SocketError, MUTEX_LOCK_ERROR_MESSAGE}, dao::main_dao};

use super::chat_room_channel::ChatRoomChannel;

#[derive(Debug)]
pub struct AppState {
    /// Rooms holds a map of roomId and the ChatRoomChannel object, pretty much just a list of all the ACTIVE chat rooms.
    /// Active chat rooms mean that there is at least one user connected that belongs to that chat room. If there are
    /// 0 participants of a certain groupconnected to a socket, the chat room must be deleted from memory.
    pub rooms: Mutex<HashMap<u32, ChatRoomChannel>>, // An id of the room & the room object (that also holds a list of all the users)
    pub connected_clients: Mutex<HashMap<SocketAddr, u32>>,
    pub user_rooms: Mutex<HashMap<u32, Vec<u32>>>, // An id of the user & a list of chat room ids
    pub conn: reqwest::Client,
    pub db_conn: MySqlPool,
}

impl AppState {
    pub fn new(db_conn: MySqlPool) -> Self {
        Self {
            rooms: Default::default(),
            connected_clients: Default::default(),
            conn: reqwest::Client::new(),
            user_rooms: Default::default(),
            db_conn
        }
    }

    pub fn add_connected_client(
        &self,
        addr: SocketAddr,
        user_id: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut user_rooms = self.user_rooms.lock().expect(MUTEX_LOCK_ERROR_MESSAGE);
        match user_rooms.insert(user_id, rooms) {
            Some(_) => Err(SocketError::boxed_error("Existing user_rooms replaced by another list of rooms, this should NOT be happening. FATAL!")),
            None => Ok(()),
        }
    }
}
