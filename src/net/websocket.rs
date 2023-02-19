use crate::{domain::state::AppState, net::handler::handle_message};
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        ConnectInfo, State,
    },
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use futures::stream::StreamExt;
use sqlx::MySqlPool;
use std::{net::SocketAddr, sync::Arc};

use super::handler::disconnect_client;

pub async fn start_ws_server(database_connection: MySqlPool, client: reqwest::Client) {
    let app_state = Arc::new(AppState::new(database_connection, client));
    let app = Router::new()
        .route("/", get(index))
        .route("/websocket", get(websocket_handler))
        .with_state(app_state);
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Finished setting up Socket server on port 3000.");
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn websocket_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket(socket, state, addr))
}

async fn websocket(stream: WebSocket, state: Arc<AppState>, addr: SocketAddr) {
    // By splitting we can send and receive at the same time.
    let (sender, mut receiver) = stream.split();
    let sender_reference = Arc::new(tokio::sync::Mutex::new(sender));

    let mut send_tasks = Vec::new();

    // Loop until a text message is found.
    while let Some(Ok(message)) = receiver.next().await {
        match handle_message(message, sender_reference.clone(), state.clone(), addr, &mut send_tasks)
            .await {
                Ok(_) => {},
                Err(error) => println!("Error recieved from handle message inside of the main websocket loop, Error: \n {}", error),
            };
    }
    match disconnect_client(&state, &addr, send_tasks).await {
        Ok(_) => {
            println!("Client disconnected.");
        }
        Err(error) => println!(
            "Error recieved from disconnect_client inside of the main websocket loop, Error: \n {}",
            error
        ),
    }
}

// Include utf-8 file at **compile** time.
async fn index() -> Html<&'static str> {
    Html(std::include_str!("../chat.html"))
}
