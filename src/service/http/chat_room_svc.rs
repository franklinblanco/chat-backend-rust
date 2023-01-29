use actix_web::{HttpRequest, web::Json};
use actix_web_utils::extensions::typed_response::TypedHttpResponse;
use chat_types::{domain::chat_room::ChatRoom, dto::chat::ChatRoomParticipants};
use dev_dtos::domain::user::user::User;
use err::MessageResource;
use reqwest::Client;
use sqlx::MySqlPool;

use crate::dao::chat_room_dao;

pub async fn get_all_user_chat_rooms(
    conn: &MySqlPool,
    _client: &Client,
    user: User,
    _request: HttpRequest,
) -> TypedHttpResponse<Vec<ChatRoom>> {
    let all_user_chat_rooms =
        match chat_room_dao::fetch_all_user_chat_rooms(conn, user.id.try_into().unwrap()).await {
            Ok(chat_rooms) => chat_rooms,
            Err(error) => {
                return TypedHttpResponse::return_standard_error(
                    400,
                    MessageResource::new_from_string(error.to_string()),
                )
            }
        };

    TypedHttpResponse::return_standard_response(200, all_user_chat_rooms)
}

pub async fn create_new_chat_room(
    conn: &MySqlPool,
    _client: &Client,
    user: User,
    _request: HttpRequest,
    participants: ChatRoomParticipants,
    title: String
) -> TypedHttpResponse<ChatRoom> {
    // Create chat room
    // Add all participants
    let mut chat_room = ChatRoom::new(title, user.id.try_into().unwrap());
    match chat_room_dao::insert_chat_room(conn, &chat_room).await {
        Ok(persisted_id) => chat_room.id = persisted_id.try_into().unwrap(),
        Err(error) => return TypedHttpResponse::return_standard_error(500, MessageResource::new_from_string(error.to_string()))
    };
    match chat_room_dao::insert_chat_room_participants(conn, participants.participants, &chat_room.id).await {
        Ok(_) => {},
        Err(error) => return TypedHttpResponse::return_standard_error(500, MessageResource::new_from_string(error.to_string()))
    }
    TypedHttpResponse::return_standard_response(200, chat_room)
}
