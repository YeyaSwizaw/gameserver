use std::net::SocketAddr;

use game::Game;

#[derive(Debug, Serialize, Deserialize)]
pub enum ProtocolMessage<G: Game> {
    ChatMessage(String),
    PlayerUpdate(G::Player),
}

#[derive(Serialize, Deserialize)]
pub enum ProtocolResponse<G: Game> {
    ChatMessage(usize, String),
    PlayerUpdate(usize, G::Player),
    Connection(usize, SocketAddr),
    ConnectionLost(usize),
}

