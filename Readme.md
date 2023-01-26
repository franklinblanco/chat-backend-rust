# Chat backend
by Franklin Blanco

This is a generic backend for all future applications that require a chat. Built with rust, and more importantly surrealDB.

#### Goals:
- [ ] Sending and recieving messages through websocket connection
- [ ] Message sending with an authentication token


### Ideal scenario
- Player requests to open a DM with another player or a League gets created 
- Player gets assigned to chat room with the help of user-svc
- League svc signals chat-svc and player device to initiate a websocket connection
- Issues player a device ID
- Player can now send and recieve messages through the connection
- Player is free to send/reciveve messages from/to any room he belongs to
- All room changes must be done by League-svc

### White paper?
- Messages must be loaded when user enters the app. 
- They should get a list of participants of every chat room they're in.
- They should have participant metadata stored or request it.

