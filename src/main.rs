mod dao;
mod domain;
mod net;
mod routes;
mod service;
mod util;

use chrono::Utc;

use crate::{
    dao::main_dao, net::websocket::start_ws_server, routes::http::main_router::start_http_server,
};

#[tokio::main]
async fn main() {
    println!("{}", Utc::now());
    dotenvy::dotenv().ok();

    let database_pool = main_dao::start_database_connection().await.unwrap();
    let client_pool = reqwest::Client::new();
    let _ = tokio::join!(
        //start_ws_server(database_pool.clone(), client_pool.clone()),
        start_http_server(database_pool.clone(), client_pool.clone())
    );
}
