extern crate lobby;
extern crate client;

#[macro_use] extern crate serde_derive;
extern crate serde;

use std::thread;
use std::io::Result;
use std::sync::{Arc, Mutex};
use std::net::{ToSocketAddrs, SocketAddr};

use lobby::{Lobby, Event, EventKind};
use client::{ProtocolMessage, ProtocolResponse};

pub struct GameServer {
    lobby: Arc<Mutex<Lobby<ProtocolMessage>>>
}

impl GameServer {
    fn new(lobby: Arc<Mutex<Lobby<ProtocolMessage>>>) -> GameServer {
        GameServer {
            lobby
        }
    }

    fn handle_event(&mut self, event: Event<ProtocolMessage>) {
        match event.event {
            EventKind::DataReceived(ProtocolMessage::ChatMessage(text)) => {
                self.lobby.lock().unwrap().send_to_except(event.from, &ProtocolResponse::ChatMessage(event.from, text.trim_right().into())).unwrap()
            },

            EventKind::ConnectionReceived(addr) => {
                self.lobby.lock().unwrap().send_to_except(event.from, &ProtocolResponse::Connection(event.from, addr)).unwrap()
            },

            EventKind::ConnectionLost(_) => self.lobby.lock().unwrap().send(&ProtocolResponse::ConnectionLost(event.from)).unwrap(),

            _ => println!("{:?}", event)
        }
    }

    pub fn spawn<A: ToSocketAddrs>(addr: A) -> Result<Arc<Mutex<GameServer>>> {
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

