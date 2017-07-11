#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate serde;

use std::io;
use std::thread;
use std::net::TcpStream;

use serde::Deserialize;
use serde_json::{Deserializer, to_writer};

use proto::{ProtocolMessage, ProtocolResponse};

mod proto;

fn handle_response(data: ProtocolResponse) {
    match data {
        ProtocolResponse::ChatMessage(from, msg) => println!("{}: {}", from, msg),
        ProtocolResponse::Connection(from, addr) => println!("{} connected from {}", from, addr),
        ProtocolResponse::ConnectionLost(from) => println!("{} disconnected", from),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ip = args[1].clone();
    let port = args[2].clone();

    let mut stream = TcpStream::connect([ip, port].join(":")).unwrap();

    let thread_stream = stream.try_clone().unwrap();
    thread_stream.set_nonblocking(false).unwrap();

    thread::spawn(move || loop {
        let mut de = Deserializer::from_reader(thread_stream.try_clone().unwrap());
        handle_response(ProtocolResponse::deserialize(&mut de).unwrap());
    });

    loop {
        let mut input = String::new();
        if let Ok(_) = io::stdin().read_line(&mut input) {
            to_writer(stream.try_clone().unwrap(), &ProtocolMessage::ChatMessage(input)).unwrap();
        }
    }
}
