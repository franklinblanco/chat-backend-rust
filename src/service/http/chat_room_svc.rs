use actix_web::HttpRequest;
use actix_web_utils::extensions::typed_response::TypedHttpResponse;
use dev_dtos::domain::user::user::User;
use err::MessageResource;
use reqwest::Client;
use sqlx::MySqlPool;

use crate::{dao::chat_room_dao, domain::chat_room::ChatRoom};

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
