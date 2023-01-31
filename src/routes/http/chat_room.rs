use actix_web::{
    get, post,
    web::{Data, Path, Json},
    HttpRequest,
};
use actix_web_utils::extensions::typed_response::TypedHttpResponse;
use chat_types::{domain::{chat_room::ChatRoom, chat_user::ChatUser}, dto::chat::ChatRoomParticipants};
use dev_macros::authenticate_route;
use reqwest::Client;
use sqlx::MySqlPool;

use crate::service::http::chat_room_svc;

#[get("/")]
pub async fn get_all_user_chat_rooms(
    conn: Data<MySqlPool>,
    client: Data<Client>,
    request: HttpRequest,
) -> TypedHttpResponse<Vec<ChatRoom>> {
    let user = authenticate_route!(request, &client);
    chat_room_svc::get_all_user_chat_rooms(&conn, &client, user, request).await
}

#[post("/{title}")]
pub async fn create_new_chat_room(
    conn: Data<MySqlPool>,
    client: Data<Client>,
    request: HttpRequest,
    title: Path<String>,
    participants: Json<ChatRoomParticipants>,    
) -> TypedHttpResponse<ChatRoom> {
    let user = authenticate_route!(request, &client);
    
    chat_room_svc::create_new_chat_room(&conn, &client, user, request, participants.0, title.to_string()).await
}

#[post("/{chat_room_id}/participants")]
pub async fn add_participants_to_chat_room(
    conn: Data<MySqlPool>,
    client: Data<Client>,
    request: HttpRequest,
    chat_room_id: Path<u32>,
    participants: Json<ChatRoomParticipants>,
) -> TypedHttpResponse<ChatRoomParticipants> {
    let user = authenticate_route!(request, &client);
    chat_room_svc::add_participants_to_chat_room(&conn, &client, user, request, participants.0, *chat_room_id).await
}

#[get("/{chat_room_id}/participants")]
pub async fn get_chat_room_participants(
    conn: Data<MySqlPool>,
    client: Data<Client>,
    request: HttpRequest,
    chat_room_id: Path<u32>,
) -> TypedHttpResponse<Vec<ChatUser>> {
    let user = authenticate_route!(request, &client);
    chat_room_svc::get_chat_room_participants(&conn, &client, user, request, *chat_room_id).await
}