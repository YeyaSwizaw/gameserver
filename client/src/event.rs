use std::net::SocketAddr;

use shared::game::Game;

pub struct Event<G: Game> {
    pub from: usize,
    pub event: EventKind<G>,
}

pub enum EventKind<G: Game> {
    Connection(SocketAddr),
    Disconnection,
    ChatMessage(String),
    PlayerUpdate(G::Player),
}

impl<G: Game> Event<G> {
    pub(crate) fn connection(from: usize, addr: SocketAddr) -> Self {
        Event {
            from,
            event: EventKind::Connection(addr),
        }
    }

    pub(crate) fn disconnection(from: usize) -> Self {
        Event {
            from,
            event: EventKind::Disconnection,
        }
    }

    pub(crate) fn chat_message<S: AsRef<str>>(from: usize, message: S) -> Self {
        Event {
            from,
            event: EventKind::ChatMessage(message.as_ref().into()),
        }
    }

    pub(crate) fn player_update(from: usize, player: G::Player) -> Self {
        Event {
            from,
            event: EventKind::PlayerUpdate(player),
        }
    }
}