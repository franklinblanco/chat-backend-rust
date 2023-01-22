use crate::{domain::{state::AppState, }, net::handler::handle_message, dao::main_dao};
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
use std::{net::SocketAddr, sync::Arc};

use super::handler::disconnect_client;

pub async fn start_ws_server() {
    println!("Setting up server.");
    let app_state = Arc::new(AppState::new(main_dao::start_database_connection().await.unwrap()));
    let app = Router::new()
        .route("/", get(index))
        .route("/websocket", get(websocket_handler))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
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
    let (mut sender, mut receiver) = stream.split();

    // Loop until a text message is found.
    while let Some(Ok(message)) = receiver.next().await {
        match handle_message(message, &mut sender, state.clone(), addr)
            .await {
                Ok(_) => {},
                Err(error) => println!("Error recieved from handle message inside of the main websocket loop, Error: \n {}", error.to_string()),
            };
    }
    match disconnect_client(&state, &addr).await {
        Ok(_) => {}
        Err(error) => println!(
            "Error recieved from disconnect_client inside of the main websocket loop, Error: \n {}",
            error.to_string()
        ),
    }
}

// Include utf-8 file at **compile** time.
async fn index() -> Html<&'static str> {
    Html(std::include_str!("../chat.html"))
}
