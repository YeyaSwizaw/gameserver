use std::{
    thread,
    io::Result,
    net::{TcpStream, ToSocketAddrs},
    sync::{
        Mutex, Arc,
        mpsc::{channel, Sender, Receiver},
    },
};

use shared::proto;
use serde::Deserialize;
use serde_json::{Deserializer, to_writer};

use crate::event::Event;

pub use shared::game::Game;
pub use shared::players::Players;

pub mod event;

pub struct GameClient<G: Game + 'static> {
    stream: TcpStream,
    players: Players<G>,

    event_rx: Receiver<Event<G>>,
}

impl<G: Game + 'static> GameClient<G> {
    fn new(stream: TcpStream, event_rx: Receiver<Event<G>>) -> Self {
        GameClient {
            stream,
            players: Players::new(),

            event_rx,
        }
    }

    fn handle_message(&mut self, tx: Sender<Event<G>>, message: proto::Server<G>) {
        match message {
            proto::Server::ChatMessage(from, message) => tx.send(Event::chat_message(from, message)).unwrap(),

            proto::Server::PlayerUpdate(from, player) => {
                self.players.update_player(from, player.clone());
                tx.send(Event::player_update(from, player)).unwrap();
            },

            proto::Server::Connection(from, addr) => {
                tx.send(Event::connection(from, addr)).unwrap();
            },

            proto::Server::ConnectionLost(from) => {
                tx.send(Event::disconnection(from)).unwrap();
            },
        }
    }

    fn send_message(&mut self, message: proto::Client<G>) {
        to_writer(self.stream.try_clone().unwrap(), &message).unwrap();
    }

    pub fn chat<S: AsRef<str>>(&mut self, message: S) {
        self.send_message(proto::Client::ChatMessage::<G>(message.as_ref().into()));
    }

    pub fn update(&mut self, player: G::Player) {
        self.send_message(proto::Client::PlayerUpdate::<G>(player));
    }

    pub fn events<'a>(&'a self) -> impl Iterator<Item=Event<G>> + 'a {
        self.event_rx.try_iter()
    }

    pub fn player(&self, id: usize) -> Option<G::Player> {
        self.players.player(id)
    }

    pub fn spawn<A: ToSocketAddrs>(addr: A) -> Result<Arc<Mutex<Self>>> {
        let stream = TcpStream::connect(addr)?;

        let thread_stream = stream.try_clone()?;
        thread_stream.set_nonblocking(false)?;

        let (tx, rx) = channel();

        let client = Arc::new(Mutex::new(GameClient::new(stream, rx)));
        let thread_client = client.clone();

        thread::spawn(move || {
            let stream = thread_stream;
            let client = thread_client;

            loop {
                let mut de = Deserializer::from_reader(stream.try_clone().unwrap());
                let message = proto::Server::<G>::deserialize(&mut de).unwrap();
                client.lock().unwrap().handle_message(tx.clone(), message);
            }
        });

        Ok(client)
    }
}
