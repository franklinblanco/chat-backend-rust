mod dao;
mod domain;
mod net;
mod routes;
mod service;
mod util;

use crate::net::websocket::start_ws_server;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    start_ws_server().await;
}
