use std::{net::{TcpListener, TcpStream}, thread::spawn};

use tungstenite::{accept, Message, WebSocket};

pub fn start_ws_server() {
    let server = TcpListener::bind("127.0.0.1:8080").unwrap();
    for stream in server.incoming() {
        spawn (move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            loop {
                let message = websocket.read_message().unwrap(); //remove match
                handle_message(&mut websocket, message);
                // We do not want to send back ping/pong messages.

            }
        });
    }
}

pub fn handle_message(websocket: &mut WebSocket<TcpStream>, message: Message) {
    if message.is_binary() {
        //serialize into your own msgs
        //websocket.write_message(message).unwrap();
    }
}

pub fn sender() {
    match  {
        
    }
}