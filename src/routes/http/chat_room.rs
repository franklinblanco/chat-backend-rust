use actix_web::{get, web::Data, HttpRequest};
use actix_web_utils::extensions::typed_response::TypedHttpResponse;
use dev_macros::authenticate_route;
use reqwest::Client;
use sqlx::MySqlPool;

use crate::{domain::chat_room::ChatRoom, service::http::chat_room_svc};

#[get("/")]
pub async fn get_all_user_chat_rooms(
    conn: Data<MySqlPool>,
    client: Data<Client>,
    request: HttpRequest,
) -> TypedHttpResponse<Vec<ChatRoom>> {
    let user = authenticate_route!(request, &client);
    chat_room_svc::get_all_user_chat_rooms(&conn, &client, user, request).await
}
