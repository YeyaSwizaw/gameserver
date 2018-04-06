use std::net::SocketAddr;

#[derive(Debug, Serialize, Deserialize)]
pub enum ProtocolMessage {
    ChatMessage(String)
}

#[derive(Serialize, Deserialize)]
pub enum ProtocolResponse {
    ChatMessage(usize, String),
    Connection(usize, SocketAddr),
    ConnectionLost(usize),
}

