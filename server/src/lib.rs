use std::{
    thread,
    io::Result,
    net::ToSocketAddrs,
    sync::{Arc, Mutex},
};

use shared::proto;
use lobby::{Lobby, Event, EventKind};
use vec_map::{Entry, VecMap};

pub use shared::game::Game;
pub use shared::players::Players;

enum Connection {
    Uninitialised,
    Player(usize)
}

pub struct GameServer<G: Game + 'static> {
    lobby: Arc<Mutex<Lobby<proto::Client<G>>>>,
    connections: Arc<Mutex<VecMap<Connection>>>,
    players: Players<G>
}

impl<G: Game + 'static> GameServer<G> {
    fn new(lobby: Arc<Mutex<Lobby<proto::Client<G>>>>) -> Self {
        let players = Players::new();

        GameServer {
            lobby,
            connections: Arc::new(Mutex::new(VecMap::new())),
            players,
        }
    }

    fn handle_event(&mut self, event: Event<proto::Client<G>>) {
        match event.event {
            EventKind::DataReceived(proto::Client::ChatMessage(text)) => {
                self.lobby.lock().unwrap().send_to_except(event.from, proto::Server::ChatMessage::<G>(event.from, text.trim_end().into())).unwrap();
            },

            EventKind::DataReceived(proto::Client::PlayerUpdate(player)) => {
                if let Entry::Occupied(ref mut entry) = self.connections.lock().unwrap().entry(event.from) {
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

                    self.lobby.lock().unwrap().send(proto::Server::PlayerUpdate::<G>(event.from, player_clone)).unwrap();
                }
            }

            EventKind::ConnectionReceived(addr) => {
                self.connections.lock().unwrap().insert(event.from, Connection::Uninitialised);

                {
                    let lobby = self.lobby.lock().unwrap();

                    lobby.send_to_except(event.from, proto::Server::Connection::<G>(event.from, addr)).unwrap();

                    self.players.with(
                        |index, player| lobby.send_to(event.from, proto::Server::PlayerUpdate::<G>(index, player.clone())).unwrap()
                    )
                }
            },

            EventKind::ConnectionLost(_) => {
                self.connections.lock().unwrap().remove(event.from);
                self.lobby.lock().unwrap().send(proto::Server::ConnectionLost::<G>(event.from)).unwrap();
            },

            EventKind::DataError(err) => {
                println!("Error: {:?}", err);
            }
        }
    }

    pub fn spawn<A: ToSocketAddrs>(addr: A) -> Result<Arc<Mutex<Self>>> {
        let lobby = Arc::new(Mutex::new(Lobby::spawn(addr)?));
        let thread_lobby = lobby.clone();

        let server = Arc::new(Mutex::new(GameServer::new(lobby)));
        let thread_server = server.clone();

        thread::spawn(move || {
            let lobby = thread_lobby;
            let server = thread_server;

            loop {
                let events: Vec<_> = {
                    let lock = lobby.lock().unwrap();
                    let iter = lock.events();
                    iter.collect()
                };

                for event in events {
                    server.lock().unwrap().handle_event(event);
                }
            }
        });

        Ok(server)
    }
}

