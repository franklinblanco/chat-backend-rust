# Chat backend
by Franklin Blanco

This is a backend for all future applications that require a chat. Built with rust

### Todo's
- Plan how HTTP and websockets will interface together? [ ]
- Think about the amount of threads you're spawning [ ]

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

