use chat_types::domain::{chat_room::ChatRoom, chat_user::ChatUser};
use chrono::Utc;
use sqlx::{mysql::MySqlQueryResult, MySqlPool};

#[allow(unused)]
pub async fn insert_chat_room(
    conn: &MySqlPool,
    chat_room: &ChatRoom,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    match sqlx::query_file!(
        "sql/chat_room/insert.sql",
        chat_room.title,
        chat_room.owner_id,
        chat_room.time_created,
        chat_room.last_updated
    )
    .execute(conn)
    .await
    {
        Ok(query_result) => Ok(query_result.last_insert_id()),
        Err(error) => Err(Box::new(error)),
    }
}

#[allow(unused)]
pub async fn get_chat_room_with_id(
    conn: &MySqlPool,
    chat_room_id: &u32,
) -> Result<Option<ChatRoom>, Box<dyn std::error::Error + Send + Sync>> {
    match sqlx::query_file_as!(ChatRoom, "sql/chat_room/get.sql", chat_room_id)
        .fetch_optional(conn)
        .await
    {
        Ok(found) => Ok(found),
        Err(error) => Err(Box::new(error)),
    }
}

#[allow(unused)]
pub async fn update_chat_room(
    conn: &MySqlPool,
    chat_room: &ChatRoom,
) -> Result<MySqlQueryResult, Box<dyn std::error::Error + Send + Sync>> {
    match sqlx::query_file!(
        "sql/chat_room/update.sql",
        chat_room.title,
        chat_room.owner_id,
        chat_room.last_updated,
        chat_room.id
    )
    .execute(conn)
    .await
    {
        Ok(query_result) => Ok(query_result),
        Err(error) => Err(Box::new(error)),
    }
}

#[allow(unused)]
pub async fn delete_chat_room(
    conn: &MySqlPool,
    chat_room_id: &u32,
) -> Result<ChatRoom, Box<dyn std::error::Error + Send + Sync>> {
    Ok(ChatRoom::default())
}

#[allow(unused)]
pub async fn fetch_all_user_chat_rooms(
    conn: &MySqlPool,
    user_id: u32,
) -> Result<Vec<ChatRoom>, Box<dyn std::error::Error + Send + Sync>> {
    match sqlx::query_file_as!(ChatRoom, "sql/chat_room/fetch_all_user_is_in.sql", user_id)
        .fetch_all(conn)
        .await
    {
        Ok(found) => Ok(found),
        Err(error) => Err(Box::new(error)),
    }
}

pub async fn insert_chat_room_participants(
    conn: &MySqlPool,
    participant_ids: &Vec<u32>,
    chat_room_id: &u32
) -> Result<MySqlQueryResult, Box<dyn std::error::Error>> {
    let time = Utc::now();
    let mut query = "INSERT INTO chat_users (
    chat_room_id,
    user_id,
    time_joined) VALUES ".to_string();
    
    for participant_id in participant_ids {
        query.push_str(&format!("({chat_room_id}, {participant_id}, \"{}\"),", time.to_string()));
    };
    
    query.remove(query.len() - 1);
    query.push_str(";");
    
    match sqlx::query(&query).execute(conn).await {
        Ok(query_result) => Ok(query_result),
        Err(error) => Err(Box::new(error)),
    }
}

pub async fn get_chat_room_participants(conn: &MySqlPool, chat_room_id: &u32) -> Result<Vec<ChatUser>, Box<dyn std::error::Error>> {
    match sqlx::query_file_as!(ChatUser, "sql/chat_users/get_all_in_chat_room.sql", chat_room_id).fetch_all(conn).await {
        Ok(chat_users) => Ok(chat_users),
        Err(error) => Err(Box::new(error)),
    }
}

pub async fn delete_chat_room_participant(conn: &MySqlPool, chat_room_id: &u32, user_id: u32) -> Result<Option<()>, Box<dyn std::error::Error>> {
    match sqlx::query_file!("sql/chat_users/remove_participant.sql", chat_room_id, user_id).execute(conn).await {
        Ok(query_result) => if query_result.rows_affected() > 0 { return Ok(Some(())) } else {return Ok(None)},
        Err(error) => Err(Box::new(error)),
    }
}