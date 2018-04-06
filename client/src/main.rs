extern crate serde_json;
extern crate serde;
extern crate server;

use std::io;
use std::thread;
use std::net::TcpStream;

use serde::Deserialize;
use serde_json::{Deserializer, to_writer};

use server::proto::{ProtocolMessage, ProtocolResponse};

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

    let stream = TcpStream::connect([ip, port].join(":")).unwrap();

    let thread_stream = stream.try_clone().unwrap();
    thread_stream.set_nonblocking(false).unwrap();

    thread::spawn(move || loop {
        let mut de = Deserializer::from_reader(thread_stream.try_clone().unwrap());
        handle_response(ProtocolResponse::deserialize(&mut de).unwrap());
    });

    loop {
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            to_writer(stream.try_clone().unwrap(), &ProtocolMessage::ChatMessage(input)).unwrap();
        }
    }
}
