use actix_web::{HttpRequest};
use actix_web_utils::extensions::typed_response::TypedHttpResponse;
use chat_types::{domain::{chat_room::ChatRoom, chat_user::ChatUser}, dto::chat::ChatRoomParticipants};
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
    match chat_room_dao::insert_chat_room_participants(conn, &participants.participants, &chat_room.id).await {
        Ok(_) => {},
        Err(error) => return TypedHttpResponse::return_standard_error(500, MessageResource::new_from_string(error.to_string()))
    }
    TypedHttpResponse::return_standard_response(200, chat_room)
}

pub async fn add_participants_to_chat_room(
    conn: &MySqlPool,
    _client: &Client,
    user: User,
    _request: HttpRequest,
    participants: ChatRoomParticipants,
    chat_room_id: u32,
) -> TypedHttpResponse<ChatRoomParticipants> {
    let persisted_chat_room = match chat_room_dao::get_chat_room_with_id(conn, &chat_room_id).await {
        Ok(persisted_chat_room_opt) => match persisted_chat_room_opt {
            Some(persisted_chat_room) => persisted_chat_room,
            None => return TypedHttpResponse::return_empty_response(404),
        },
        Err(error) => return TypedHttpResponse::return_standard_error(500, MessageResource::new_from_string(error.to_string())),
    };
    if persisted_chat_room.owner_id != user.id as u32 {
        return TypedHttpResponse::return_standard_error(401, MessageResource::new_from_str("User requesting to add participants to chat room isn't the owner of the chat room..."));
    }
    let persisted_chat_room_participants = match chat_room_dao::get_chat_room_participants(conn, &chat_room_id).await {
        Ok(persisted_chat_room_participants) => persisted_chat_room_participants,
        Err(error) => return TypedHttpResponse::return_standard_error(500, MessageResource::new_from_string(error.to_string())),
    };
    if persisted_chat_room_participants.iter().any(|participant| participants.participants.contains(&participant.user_id)) {
        return TypedHttpResponse::return_standard_error(400, MessageResource::new_from_str("At least one of the participants in the list to add is already in this chat room."));
    };
    match chat_room_dao::insert_chat_room_participants(conn, &participants.participants, &chat_room_id).await {
        Ok(_) => {},
        Err(error) => return TypedHttpResponse::return_standard_error(500, MessageResource::new_from_string(error.to_string()))
    }
    TypedHttpResponse::return_standard_response(200, participants)
}

pub async fn get_chat_room_participants(
    conn: &MySqlPool,
    _client: &Client,
    user: User,
    _request: HttpRequest,
    chat_room_id: u32,
) -> TypedHttpResponse<Vec<ChatUser>> {
    match chat_room_dao::get_chat_room_participants(conn, &chat_room_id).await {
        Ok(participants) => TypedHttpResponse::return_standard_response(200, participants),
        Err(error) => TypedHttpResponse::return_standard_error(500, MessageResource::new_from_string(error.to_string()))
    }
}