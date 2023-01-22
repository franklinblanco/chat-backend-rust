use tokio::sync::broadcast;

use super::chat_message::ChatMessage;

#[derive(Debug, Clone)]
pub struct ChatRoomChannel {
    /// Sender es un canal que permite a muchos receptores consumir el mismo flujo de datos.
    /// Se usa para transmitir mensajes a muchos receptores. Tambien se usa para enviar mensajes al canal broadcast
    /// y se pueden crear cualquier número de Receptores<> con el metodo sender.subscribe() desde el canal de broadcast
    /// para recibir los mensajes.
    /// Lo mas importante aqui es que esto sirve para elegir a quien se le va a enviar mensajes.
    /// El Tipo dentro del Sender es lo que se va a enviar a traves de los canales
    pub recipient_sockets: broadcast::Sender<String>, // TODO: Evaluate customer & employee (maybe 2 different ones)
    pub participants: Vec<u32>,
    /// El id en la base de datos de este chat room
    pub chat_room_id: u32, //TODO: replace with chat room domain obj
    pub messages: Vec<ChatMessage>,
}
