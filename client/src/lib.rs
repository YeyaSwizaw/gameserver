#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate serde;

mod proto;

pub use proto::{ProtocolMessage, ProtocolResponse};
