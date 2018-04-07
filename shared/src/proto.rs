use std::net::SocketAddr;

use game::Game;

pub use self::{
    server::Protocol as Server,
    client::Protocol as Client,
};

mod server {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Protocol<G: Game> {
        ChatMessage(usize, String),
        PlayerUpdate(usize, G::Player),
        Connection(usize, SocketAddr),
        ConnectionLost(usize),
    }

}

mod client {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Protocol<G: Game> {
        ChatMessage(String),
        PlayerUpdate(G::Player),
    }
}


