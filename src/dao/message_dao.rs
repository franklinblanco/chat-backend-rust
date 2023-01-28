use sqlx::{mysql::MySqlQueryResult, MySqlPool};

use crate::domain::chat_message::ChatMessage;

pub async fn get_message(
    conn: &MySqlPool,
    message_id: &u32,
) -> Result<Option<ChatMessage>, Box<dyn std::error::Error + Send + Sync>> {
    match sqlx::query_file_as!(ChatMessage, "sql/message/get.sql", message_id)
        .fetch_optional(conn)
        .await
    {
        Ok(found) => Ok(found),
        Err(error) => Err(Box::new(error)),
    }
}

pub async fn insert_message(
    conn: &MySqlPool,
    message: &ChatMessage,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    match sqlx::query_file!(
        "sql/message/insert.sql",
        message.from_id,
        message.to_id,
        message.message,
        message.time_sent,
        message.time_delivered,
        message.time_seen
    )
    .execute(conn)
    .await
    {
        Ok(query_result) => Ok(query_result.last_insert_id()),
        Err(error) => Err(Box::new(error)),
    }
}

pub async fn update_message(
    conn: &MySqlPool,
    message: &ChatMessage,
) -> Result<MySqlQueryResult, Box<dyn std::error::Error + Send + Sync>> {
    match sqlx::query_file!(
        "sql/message/update.sql",
        message.time_delivered,
        message.time_seen,
        message.id
    )
    .execute(conn)
    .await
    {
        Ok(query_result) => Ok(query_result),
        Err(error) => Err(Box::new(error)),
    }
}

pub async fn fetch_messages_with_ids(
    conn: &MySqlPool,
    message_ids: &Vec<u32>,
) -> Result<Vec<ChatMessage>, Box<dyn std::error::Error + Send + Sync>> {
    let mut query = "SELECT * FROM message where id in (".to_string();
    for message_id in message_ids {
        query.push_str(&format!("{},", message_id.to_string()));
    }
    query.remove(query.len() - 1);
    query.push_str(");");
    match sqlx::query_as(query.as_str()).fetch_all(conn).await {
        Ok(messages) => Ok(messages),
        Err(error) => Err(Box::new(error)),
    }
}
