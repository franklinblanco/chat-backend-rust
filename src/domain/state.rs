use std::{collections::HashMap, net::SocketAddr, sync::Mutex};

use super::chat_room_channel::ChatRoomChannel;

#[derive(Debug)]
pub struct AppState {
    /// Rooms holds a map of roomId and the ChatRoomChannel object, pretty much just a list of all the ACTIVE chat rooms.
    /// Active chat rooms mean that there is at least one user connected that belongs to that chat room. If there are
    /// 0 participants of a certain groupconnected to a socket, the chat room must be deleted from memory.
    pub rooms: Mutex<HashMap<i32, ChatRoomChannel>>, // An id of the room & the room object (that also holds a list of all the users)
    pub connected_clients: Mutex<HashMap<SocketAddr, u32>>,
    pub user_rooms: Mutex<HashMap<u32, Vec<i32>>>, // An id of the user & a list of chat room ids
                                                   //pub conn: reqwest::Client,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            rooms: Default::default(),
            connected_clients: Default::default(),
            /*conn: client::initialize_client()*/ user_rooms: Default::default(),
        }
    }
}
