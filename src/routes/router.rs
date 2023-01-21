const _SUBSCRIBED_ROUTES: [MainRouter; 8] = [
    MainRouter::ChatActions(ChatActionRouter::SendMessage),
    MainRouter::ChatActions(ChatActionRouter::GetUpdates),
    MainRouter::RoomActions(RoomActionRouter::CreateRoom),
    MainRouter::RoomActions(RoomActionRouter::ExitRoom),
    MainRouter::RoomActions(RoomActionRouter::KickUserFromRoom),
    MainRouter::RoomActions(RoomActionRouter::EscalateUser),
    MainRouter::RoomActions(RoomActionRouter::AddUserToRoom),
    MainRouter::PlatformActions(PlatformActionRouter::CreatePlatform),
];

#[allow(unused)]
pub enum ChatActionRouter {
    SendMessage,
    GetUpdates,
}
#[allow(unused)]
pub enum RoomActionRouter {
    CreateRoom,
    ExitRoom,
    KickUserFromRoom,
    EscalateUser,
    AddUserToRoom,
}
#[allow(unused)]
pub enum PlatformActionRouter {
    CreatePlatform,
}
#[allow(unused)]
pub enum MainRouter {
    ChatActions(ChatActionRouter),
    PlatformActions(PlatformActionRouter),
    RoomActions(RoomActionRouter),
}
