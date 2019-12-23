use std::{
    thread,
    io::Result,
    net::ToSocketAddrs,
    sync::{Arc, Mutex},
};

use shared::proto;
use lobby::{Lobby, LobbySender, Event, EventKind};
use vec_map::{Entry, VecMap};

pub use shared::game::Game;
pub use shared::players::Players;

enum Connection {
    Uninitialised,
    Player(usize)
}

pub struct GameServer<G: Game + 'static> {
    sender: LobbySender,
    connections: VecMap<Connection>,
    players: Players<G>,
}

impl<G: Game + 'static> GameServer<G> {
    fn new(sender: LobbySender) -> Self {
        GameServer {
            sender,
            connections: VecMap::new(),
            players: Players::new(),
        }
    }

    fn handle_event(&mut self, event: Event<proto::Client<G>>) {
        match event.event {
            EventKind::DataReceived(proto::Client::ChatMessage(text)) => {
                self.sender.send_to_except(event.from, proto::Server::ChatMessage::<G>(event.from, text.trim_end().into())).unwrap();
            },

            EventKind::DataReceived(proto::Client::PlayerUpdate(player)) => {
                if let Entry::Occupied(ref mut entry) = self.connections.entry(event.from) {
                    let player_clone = player.clone();
                    let mut players = self.players.clone();

                    take_mut::take(
                        entry.get_mut(),
                        move |connection| match connection {
                            Connection::Uninitialised => Connection::Player(players.add_player(player)),
                            
                            Connection::Player(player_index) => {
                                players.update_player(player_index, player);
                                connection
                            }
                        });

                    self.sender.send(proto::Server::PlayerUpdate::<G>(event.from, player_clone)).unwrap();
                }
            }

            EventKind::ConnectionReceived(addr) => {
                self.connections.insert(event.from, Connection::Uninitialised);

                {
                    self.sender.send_to_except(event.from, proto::Server::Connection::<G>(event.from, addr)).unwrap();

                    self.players.with(
                        |index, player| self.sender.send_to(event.from, proto::Server::PlayerUpdate::<G>(index, player.clone())).unwrap()
                    )
                }
            },

            EventKind::ConnectionLost(_) => {
                self.connections.remove(event.from);
                self.sender.send(proto::Server::ConnectionLost::<G>(event.from)).unwrap();
            },

            EventKind::DataError(err) => {
                println!("Error: {:?}", err);
            }
        }
    }

    pub fn spawn<A: ToSocketAddrs>(addr: A) -> Result<Arc<Mutex<Self>>> {
        let (sender, receiver) = Lobby::<proto::Client<G>>::spawn(addr).unwrap();

        let server = Arc::new(Mutex::new(GameServer::new(sender)));
        let thread_server = server.clone();

        thread::spawn(move || {
            let server = thread_server;

            loop {
                for event in receiver.events() {
                    server.lock().unwrap().handle_event(event);
                }
            }
        });

        Ok(server)
    }
}

