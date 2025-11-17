use common::protocol::ServerMessage;

#[derive(Debug)]
pub enum UiMessage {
    JoinRoom { address: String, username: String },
    SendChat { text: String },
}

#[derive(Debug)]
pub enum NetworkMessage {
    InvalidAddress,
    ServerMessage(ServerMessage),
}
