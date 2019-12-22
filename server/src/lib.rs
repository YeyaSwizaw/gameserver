extern crate lobby;
extern crate shared;
extern crate vec_map;

use std::{
    thread,
    io::Result,
    net::ToSocketAddrs,
    sync::{Arc, Mutex},
};

use shared::proto;
use lobby::{Lobby, Event, EventKind};
use vec_map::VecMap;

pub use shared::game::Game;

pub struct GameServer<G: Game + 'static> {
    lobby: Arc<Mutex<Lobby<proto::Client<G>>>>,
    players: Arc<Mutex<VecMap<G::Player>>>,
}

impl<G: Game + 'static> GameServer<G> {
    fn new(lobby: Arc<Mutex<Lobby<proto::Client<G>>>>) -> Self {
        GameServer {
            lobby,
            players: Arc::new(Mutex::new(VecMap::new())),
        }
    }

    fn handle_event(&mut self, event: Event<proto::Client<G>>) {
        match event.event {
            EventKind::DataReceived(proto::Client::ChatMessage(text)) => {
                self.lobby.lock().unwrap().send_to_except(event.from, proto::Server::ChatMessage::<G>(event.from, text.trim_end().into())).unwrap();
            },

            EventKind::DataReceived(proto::Client::PlayerUpdate(player)) => {
                self.players.lock().unwrap().insert(event.from, player.clone());
                self.lobby.lock().unwrap().send(proto::Server::PlayerUpdate::<G>(event.from, player)).unwrap();
            }

            EventKind::ConnectionReceived(addr) => {
                self.players.lock().unwrap().insert(event.from, Default::default());

                {
                    let lobby = self.lobby.lock().unwrap();
                    lobby.send_to_except(event.from, proto::Server::Connection::<G>(event.from, addr)).unwrap();

                    for (id, player) in self.players.lock().unwrap().iter() {
                        lobby.send_to(event.from, proto::Server::PlayerUpdate::<G>(id, player.clone())).unwrap();
                    }
                }
            },

            EventKind::ConnectionLost(_) => {
                self.players.lock().unwrap().remove(event.from);
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

