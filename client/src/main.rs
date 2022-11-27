use tungstenite::{connect, Message};
use url::Url;

fn main() {

    let (mut socket, response) =
        connect(Url::parse("ws://localhost:8080/socket").unwrap()).expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    socket.write_message(Message::Text("Hello WebSocket".into())).unwrap();
    loop {
        println!("loop ran");
        let msg = socket.read_message().expect("Error reading message");
        println!("Received: {}", msg);
    }
    // socket.close(None);
}