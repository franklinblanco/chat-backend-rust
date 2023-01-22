use sqlx::{MySqlPool, mysql::MySqlQueryResult};

use crate::domain::chat_room::ChatRoom;

pub async fn insert_chat_room(conn: &MySqlPool, chat_room: &ChatRoom) -> Result<u64, Box<dyn std::error::Error>> {
    match sqlx::query_file!("sql/chat_room/insert.sql", chat_room.title, chat_room.owner_id, chat_room.time_created, chat_room.last_updated).execute(conn).await {
        Ok(query_result) => Ok(query_result.last_insert_id()),
        Err(error) => Err(Box::new(error)),
    }
}

pub async fn get_chat_room_with_id(conn: &MySqlPool, chat_room_id: &u32) -> Result<Option<ChatRoom>, Box<dyn std::error::Error>> {
    match sqlx::query_file_as!(ChatRoom, "sql/chat_room/get.sql", chat_room_id).fetch_optional(conn).await {
        Ok(found) => Ok(found),
        Err(error) => Err(Box::new(error)),
    }
}

pub async fn update_chat_room(conn: &MySqlPool, chat_room: &ChatRoom) -> Result<MySqlQueryResult, Box<dyn std::error::Error>> {
    match sqlx::query_file!("sql/chat_room/update.sql", chat_room.title, chat_room.owner_id, chat_room.last_updated, chat_room.id).execute(conn).await {
        Ok(query_result) => Ok(query_result),
        Err(error) => Err(Box::new(error)),
    }
}

pub async fn delete_chat_room(conn: &MySqlPool, chat_room_id: &u32) -> Result<ChatRoom, Box<dyn std::error::Error>> {
    Ok(ChatRoom::default())
}

pub async fn fetch_all_user_chat_rooms(conn: &MySqlPool, user_id: u32) -> Result<Vec<ChatRoom>, Box<dyn std::error::Error>> {
    Ok(Vec::new())
}