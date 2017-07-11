extern crate server;

use server::GameServer;

fn main() {
    let server = GameServer::spawn("127.0.0.1:8080").unwrap();
    loop {}
}
