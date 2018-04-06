#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate server;

use std::io::{self, Write};
use std::thread;
use std::net::TcpStream;

use serde::Deserialize;
use serde_json::{Deserializer, to_writer};

use server::proto::{ProtocolMessage, ProtocolResponse};
use game::{TestGame, Player};

mod game;

fn handle_response(data: ProtocolResponse<TestGame>) {
    match data {
        ProtocolResponse::ChatMessage(from, msg) => println!("{}: {}", from, msg),
        ProtocolResponse::PlayerUpdate(from, player) => println!("{}: {:?}", from, player),
        ProtocolResponse::Connection(from, addr) => println!("{} connected from {}", from, addr),
        ProtocolResponse::ConnectionLost(from) => println!("{} disconnected", from),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ip = args[1].clone();
    let port = args[2].clone();

    let stream = TcpStream::connect([ip, port].join(":")).unwrap();

    let thread_stream = stream.try_clone().unwrap();
    thread_stream.set_nonblocking(false).unwrap();

    print!("Enter Name:");
    io::stdout().flush().ok().expect("Could not flush stdout");

    let name = {
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            input
        }
        else {
            panic!("Unable to read name from std")
        }
    };

    thread::spawn(move || loop {
        let mut de = Deserializer::from_reader(thread_stream.try_clone().unwrap());
        handle_response(ProtocolResponse::deserialize(&mut de).unwrap());
    });

    to_writer(stream.try_clone().unwrap(), &ProtocolMessage::PlayerUpdate::<TestGame>(Player::new(name))).unwrap();

    loop {
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            to_writer(stream.try_clone().unwrap(), &ProtocolMessage::ChatMessage::<TestGame>(input)).unwrap();
        }
    }
}
