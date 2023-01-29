use actix_web::{
    get, post,
    web::{Data, Path},
    HttpRequest,
};
use actix_web_utils::extensions::typed_response::TypedHttpResponse;
use chat_types::domain::chat_room::ChatRoom;
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
) -> TypedHttpResponse<ChatRoom> {
    let user = authenticate_route!(request, &client);
    TypedHttpResponse::return_empty_response(200)
}
