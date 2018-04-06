extern crate lobby;

#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate vec_map;

use std::thread;
use std::io::Result;
use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};

use vec_map::VecMap;

use lobby::{Lobby, Event, EventKind};
use proto::{ProtocolMessage, ProtocolResponse};
use game::Game;

pub mod proto;
pub mod game;

pub struct GameServer<G: Game + 'static> {
    lobby: Arc<Mutex<Lobby<ProtocolMessage<G>>>>,
    players: Arc<Mutex<VecMap<G::Player>>>,
}

impl<G: Game + 'static> GameServer<G> {
    fn new(lobby: Arc<Mutex<Lobby<ProtocolMessage<G>>>>) -> Self {
        GameServer {
            lobby,
            players: Arc::new(Mutex::new(VecMap::new())),
        }
    }

    fn handle_event(&mut self, event: Event<ProtocolMessage<G>>) {
        match event.event {
            EventKind::DataReceived(ProtocolMessage::ChatMessage(text)) => {
                self.lobby.lock().unwrap().send_to_except(event.from, ProtocolResponse::ChatMessage::<G>(event.from, text.trim_right().into())).unwrap();
            },

            EventKind::DataReceived(ProtocolMessage::PlayerUpdate(player)) => {
                self.players.lock().unwrap().insert(event.from, player.clone());
                self.lobby.lock().unwrap().send_to_except(event.from, ProtocolResponse::PlayerUpdate::<G>(event.from, player)).unwrap();
            }

            EventKind::ConnectionReceived(addr) => {
                self.players.lock().unwrap().insert(event.from, Default::default());
                self.lobby.lock().unwrap().send_to_except(event.from, ProtocolResponse::Connection::<G>(event.from, addr)).unwrap();
            },

            EventKind::ConnectionLost(_) => {
                self.players.lock().unwrap().remove(event.from);
                self.lobby.lock().unwrap().send(ProtocolResponse::ConnectionLost::<G>(event.from)).unwrap();
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

