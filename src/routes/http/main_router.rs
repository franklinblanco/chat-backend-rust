use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use sqlx::MySqlPool;

use crate::{routes::http::chat_room::{get_all_user_chat_rooms, create_new_chat_room, add_participants_to_chat_room, get_chat_room_participants, leave_chat_room, kick_user_from_chat_room}};

pub async fn start_http_server(
    database_connection: MySqlPool,
    client: reqwest::Client,
) -> Result<(), std::io::Error> {
    let db_state = web::Data::new(database_connection.clone());
    let client_state = web::Data::new(client);
    let server_future = HttpServer::new(move || {
        let cors_policy = Cors::permissive();
        App::new()
            .wrap(cors_policy)
            //  Define routes & pass in shared state
            .app_data(db_state.clone())
            .app_data(client_state.clone())
            .service(
                web::scope("/chat")
                    .service(web::scope("/room")
                        .service(get_all_user_chat_rooms)
                        .service(create_new_chat_room)
                        .service(add_participants_to_chat_room)
                        .service(get_chat_room_participants)
                        .service(leave_chat_room)
                        .service(kick_user_from_chat_room))
                    .service(
                        web::scope("/messages"), //    .service()
                    ),
            )
    });
    println!("Finished HTTP server setup on port 8082.");
    return server_future.bind(("0.0.0.0", 8082))?.run().await;
}
