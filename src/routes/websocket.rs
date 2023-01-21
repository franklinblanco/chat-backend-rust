// These routes are only for the client to use, these should have VERY Limited use,
// As they don't go through any external services, they only validate the initial connection,
// after that the user gets free reign over these routes

pub async fn send_message_to_chatroom() {}

pub async fn close_websocket_connection() {}
