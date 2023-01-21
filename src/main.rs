use routes::start_ws_server;

mod dao;
mod domain;
mod routes;

#[tokio::main]
async fn main() {
    start_ws_server().await;
}
